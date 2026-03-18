use std::{fs, path::PathBuf};

use serde_json::{json, Map, Value};
use thiserror::Error;

use sysinfo::{ProcessesToUpdate, System};

use crate::models::config::{HandyConfig, HandyConfigStatus, ProviderConfig};
use crate::utils::paths::{
    handy_config_path, latest_handy_config_backup_path, new_handy_config_backup_path,
};

const OLLAMA_PROVIDER_ID: &str = "ollama-local";
const HANDY_CUSTOM_PROVIDER_ID: &str = "custom";
const HANDY_PROMPT_ID: &str = "handy_launcher_optimized";
const HANDY_PROMPT_NAME: &str = "Handy Launcher Optimized";
const HANDY_PROMPT_TEXT: &str =
    "You are post-processing voice transcription for Handy. Correct grammar, punctuation, and capitalization while preserving the speaker's meaning and tone. Do not add new facts or change intent.";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config path not available")]
    MissingPath,
    #[error("Handy is running. Close Handy before updating its settings.")]
    HandyRunning,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn status() -> Result<HandyConfigStatus, ConfigError> {
        let config_path = handy_config_path();
        let config_exists = config_path
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false);
        let root = Self::read_settings_json()?;
        let settings = Self::settings_scope(&root);

        let current_provider_id = settings
            .get("post_process_provider_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let configured_model = settings
            .get("post_process_models")
            .and_then(|models| models.get(HANDY_CUSTOM_PROVIDER_ID))
            .and_then(Value::as_str)
            .map(str::to_string);
        let selected_prompt_id = settings
            .get("post_process_selected_prompt_id")
            .and_then(Value::as_str)
            .map(str::to_string);

        Ok(HandyConfigStatus {
            config_path: config_path.map(|path| path.display().to_string()),
            latest_backup_path: latest_handy_config_backup_path()
                .map(|path| path.display().to_string()),
            config_exists,
            handy_running: Self::is_handy_running(),
            current_provider_id,
            configured_model,
            selected_prompt_id,
        })
    }

    pub fn read_config() -> Result<HandyConfig, ConfigError> {
        if let Some(path) = handy_config_path() {
            if !path.exists() {
                return Ok(HandyConfig::default());
            }

            let contents = fs::read_to_string(&path)?;
            let config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(HandyConfig::default())
        }
    }

    pub fn read_settings_json() -> Result<Value, ConfigError> {
        match handy_config_path() {
            Some(path) if path.exists() => {
                let contents = fs::read_to_string(path)?;
                Ok(serde_json::from_str(&contents)?)
            }
            Some(_) => Ok(json!({})),
            None => Err(ConfigError::MissingPath),
        }
    }

    pub fn write_config(config: &HandyConfig) -> Result<(), ConfigError> {
        let backup_path = Self::backup_config()?;
        match Self::write_json_value(&serde_json::to_value(config)?) {
            Ok(()) => Ok(()),
            Err(err) => {
                if let Some(path) = backup_path.as_ref() {
                    let _ = Self::restore_backup(path);
                }
                Err(err)
            }
        }
    }

    pub fn write_json_value(value: &Value) -> Result<(), ConfigError> {
        let path = handy_config_path().ok_or(ConfigError::MissingPath)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, serde_json::to_string_pretty(value)?)?;
        fs::rename(temp_path, path)?;
        Ok(())
    }

    pub fn backup_config() -> Result<Option<PathBuf>, ConfigError> {
        let path = handy_config_path().ok_or(ConfigError::MissingPath)?;
        if !path.exists() {
            return Ok(None);
        }

        let backup = new_handy_config_backup_path().ok_or(ConfigError::MissingPath)?;
        if let Some(parent) = backup.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&path, &backup)?;

        Ok(Some(backup))
    }

    pub fn restore_backup(backup: &PathBuf) -> Result<(), ConfigError> {
        let path = handy_config_path().ok_or(ConfigError::MissingPath)?;
        if backup.exists() {
            fs::copy(backup, &path)?;
        }

        Ok(())
    }

    pub fn merge_ollama_config(config: &mut HandyConfig, model_name: &str, port: u16) {
        config.post_process_provider_id = OLLAMA_PROVIDER_ID.to_string();
        config
            .post_process_models
            .insert("ollama".into(), model_name.to_string());

        let provider = ProviderConfig {
            id: OLLAMA_PROVIDER_ID.to_string(),
            model: model_name.to_string(),
            endpoint: Some(format!("http://127.0.0.1:{}", port)),
            extra: Default::default(),
        };

        config
            .post_process_providers
            .insert(OLLAMA_PROVIDER_ID.to_string(), provider);
    }

    pub fn configure_handy_with_ollama(model_name: &str, port: u16) -> Result<Value, ConfigError> {
        if Self::is_handy_running() {
            return Err(ConfigError::HandyRunning);
        }

        let mut root = Self::read_settings_json()?;
        Self::merge_ollama_settings(Self::settings_scope_mut(&mut root), model_name, port);
        let backup_path = Self::backup_config()?;

        match Self::write_json_value(&root) {
            Ok(()) => Ok(root),
            Err(err) => {
                if let Some(path) = backup_path.as_ref() {
                    let _ = Self::restore_backup(path);
                }
                Err(err)
            }
        }
    }

    pub fn merge_ollama_settings(settings: &mut Value, model_name: &str, port: u16) {
        let settings_object = Self::ensure_object(settings);
        settings_object.insert(
            "post_process_provider_id".into(),
            Value::String(HANDY_CUSTOM_PROVIDER_ID.into()),
        );
        settings_object.insert(
            "post_process_selected_prompt_id".into(),
            Value::String(HANDY_PROMPT_ID.into()),
        );

        Self::upsert_custom_provider(settings_object, port);
        Self::upsert_custom_model(settings_object, model_name);
        Self::upsert_prompt(settings_object);
    }

    fn settings_scope_mut(root: &mut Value) -> &mut Value {
        if root.get("settings").is_some() {
            let root_object = Self::ensure_object(root);
            let settings = root_object
                .entry("settings")
                .or_insert_with(|| Value::Object(Map::new()));
            if !settings.is_object() {
                *settings = Value::Object(Map::new());
            }
            settings
        } else {
            root
        }
    }

    fn settings_scope(root: &Value) -> &Value {
        root.get("settings").unwrap_or(root)
    }

    fn is_handy_running() -> bool {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        system.processes().values().any(|process| {
            let name = process.name().to_string_lossy().to_lowercase();
            name.contains("handy") && !name.contains("launcher")
        })
    }

    fn upsert_custom_provider(settings: &mut Map<String, Value>, port: u16) {
        let base_url = format!("http://127.0.0.1:{port}/v1");

        match settings.get_mut("post_process_providers") {
            Some(Value::Object(providers)) => {
                let provider =
                    Self::ensure_child_object(providers, HANDY_CUSTOM_PROVIDER_ID.to_string());
                provider.insert("id".into(), Value::String(HANDY_CUSTOM_PROVIDER_ID.into()));
                provider.insert("label".into(), Value::String("Custom".into()));
                provider.insert("base_url".into(), Value::String(base_url));
                provider
                    .entry("api_key")
                    .or_insert(Value::String("ollama".into()));
                provider.insert("models_endpoint".into(), Value::String("/models".into()));
                provider
                    .entry("allow_base_url_edit")
                    .or_insert(Value::Bool(true));
                provider
                    .entry("supports_structured_output")
                    .or_insert(Value::Bool(false));
            }
            Some(Value::Array(providers)) => {
                let provider_index = providers
                    .iter()
                    .position(|entry| {
                        entry.get("id").and_then(Value::as_str) == Some(HANDY_CUSTOM_PROVIDER_ID)
                    })
                    .unwrap_or_else(|| {
                        providers.push(Value::Object(Map::new()));
                        providers.len() - 1
                    });
                let provider = Self::ensure_object(&mut providers[provider_index]);
                provider.insert("id".into(), Value::String(HANDY_CUSTOM_PROVIDER_ID.into()));
                provider.insert("label".into(), Value::String("Custom".into()));
                provider.insert("base_url".into(), Value::String(base_url));
                provider
                    .entry("api_key")
                    .or_insert(Value::String("ollama".into()));
                provider.insert("models_endpoint".into(), Value::String("/models".into()));
                provider
                    .entry("allow_base_url_edit")
                    .or_insert(Value::Bool(true));
                provider
                    .entry("supports_structured_output")
                    .or_insert(Value::Bool(false));
            }
            _ => {
                settings.insert(
                    "post_process_providers".into(),
                    json!({
                        HANDY_CUSTOM_PROVIDER_ID: {
                            "id": HANDY_CUSTOM_PROVIDER_ID,
                            "label": "Custom",
                            "base_url": base_url,
                            "api_key": "ollama",
                            "models_endpoint": "/models",
                            "allow_base_url_edit": true,
                            "supports_structured_output": false
                        }
                    }),
                );
            }
        }
    }

    fn upsert_custom_model(settings: &mut Map<String, Value>, model_name: &str) {
        let models = match settings.get_mut("post_process_models") {
            Some(Value::Object(models)) => models,
            _ => {
                settings.insert("post_process_models".into(), Value::Object(Map::new()));
                match settings.get_mut("post_process_models") {
                    Some(Value::Object(models)) => models,
                    _ => unreachable!("post_process_models inserted as object"),
                }
            }
        };

        models.insert(
            HANDY_CUSTOM_PROVIDER_ID.into(),
            Value::String(model_name.to_string()),
        );
    }

    fn upsert_prompt(settings: &mut Map<String, Value>) {
        match settings.get_mut("post_process_prompts") {
            Some(Value::Object(prompts)) => {
                let prompt = Self::ensure_child_object(prompts, HANDY_PROMPT_ID.to_string());
                prompt.insert("id".into(), Value::String(HANDY_PROMPT_ID.into()));
                prompt.insert("name".into(), Value::String(HANDY_PROMPT_NAME.into()));
                prompt.insert("prompt".into(), Value::String(HANDY_PROMPT_TEXT.into()));
            }
            Some(Value::Array(prompts)) => {
                let prompt_index = prompts
                    .iter()
                    .position(|entry| {
                        entry.get("id").and_then(Value::as_str) == Some(HANDY_PROMPT_ID)
                    })
                    .unwrap_or_else(|| {
                        prompts.push(Value::Object(Map::new()));
                        prompts.len() - 1
                    });
                let prompt = Self::ensure_object(&mut prompts[prompt_index]);
                prompt.insert("id".into(), Value::String(HANDY_PROMPT_ID.into()));
                prompt.insert("name".into(), Value::String(HANDY_PROMPT_NAME.into()));
                prompt.insert("prompt".into(), Value::String(HANDY_PROMPT_TEXT.into()));
            }
            _ => {
                settings.insert(
                    "post_process_prompts".into(),
                    json!({
                        HANDY_PROMPT_ID: {
                            "id": HANDY_PROMPT_ID,
                            "name": HANDY_PROMPT_NAME,
                            "prompt": HANDY_PROMPT_TEXT
                        }
                    }),
                );
            }
        }
    }

    fn ensure_object(value: &mut Value) -> &mut Map<String, Value> {
        if !value.is_object() {
            *value = Value::Object(Map::new());
        }

        match value {
            Value::Object(map) => map,
            _ => unreachable!("value replaced with object"),
        }
    }

    fn ensure_child_object<'a>(
        parent: &'a mut Map<String, Value>,
        key: String,
    ) -> &'a mut Map<String, Value> {
        let child = parent
            .entry(key)
            .or_insert_with(|| Value::Object(Map::new()));
        Self::ensure_object(child)
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigManager;
    use crate::models::config::{HandyConfig, ProviderConfig};
    use serde_json::json;

    #[test]
    fn merge_ollama_config_preserves_existing_non_ollama_settings() {
        let mut config = HandyConfig::default();
        config.post_process_selected_prompt_id = "proofread".into();
        config
            .post_process_prompts
            .insert("proofread".into(), "Keep punctuation".into());
        config.post_process_providers.insert(
            "openai".into(),
            ProviderConfig {
                id: "openai".into(),
                model: "gpt-4o-mini".into(),
                endpoint: Some("https://api.openai.com/v1".into()),
                extra: Default::default(),
            },
        );

        ConfigManager::merge_ollama_config(&mut config, "llama3.2:3b", 12400);

        assert_eq!(config.post_process_provider_id, "ollama-local");
        assert_eq!(config.post_process_selected_prompt_id, "proofread");
        assert_eq!(
            config.post_process_models.get("ollama").map(String::as_str),
            Some("llama3.2:3b")
        );
        assert!(config.post_process_providers.contains_key("openai"));
        assert_eq!(
            config
                .post_process_providers
                .get("ollama-local")
                .and_then(|provider| provider.endpoint.as_deref()),
            Some("http://127.0.0.1:12400")
        );
    }

    #[test]
    fn merge_ollama_settings_updates_object_shaped_handy_config() {
        let mut settings = json!({
            "post_process_provider_id": "openai",
            "post_process_providers": {
                "custom": {
                    "id": "custom",
                    "label": "Custom",
                    "api_key": "persist-me"
                }
            },
            "post_process_models": {},
            "post_process_prompts": {},
            "post_process_selected_prompt_id": "existing"
        });

        ConfigManager::merge_ollama_settings(&mut settings, "llama3.2:3b", 12400);

        assert_eq!(settings["post_process_provider_id"], "custom");
        assert_eq!(
            settings["post_process_providers"]["custom"]["base_url"],
            "http://127.0.0.1:12400/v1"
        );
        assert_eq!(
            settings["post_process_providers"]["custom"]["api_key"],
            "persist-me"
        );
        assert_eq!(settings["post_process_models"]["custom"], "llama3.2:3b");
        assert_eq!(
            settings["post_process_selected_prompt_id"],
            "handy_launcher_optimized"
        );
        assert_eq!(
            settings["post_process_prompts"]["handy_launcher_optimized"]["id"],
            "handy_launcher_optimized"
        );
    }

    #[test]
    fn merge_ollama_settings_updates_array_shaped_handy_config() {
        let mut settings = json!({
            "post_process_provider_id": "openai",
            "post_process_providers": [
                {
                    "id": "custom",
                    "label": "Custom",
                    "api_key": "persist-me"
                }
            ],
            "post_process_models": {},
            "post_process_prompts": [
                {
                    "id": "existing",
                    "name": "Existing",
                    "prompt": "Keep me"
                }
            ],
            "post_process_selected_prompt_id": "existing"
        });

        ConfigManager::merge_ollama_settings(&mut settings, "llama3.2:3b", 12400);

        assert_eq!(settings["post_process_provider_id"], "custom");
        assert_eq!(
            settings["post_process_providers"][0]["base_url"],
            "http://127.0.0.1:12400/v1"
        );
        assert_eq!(
            settings["post_process_providers"][0]["api_key"],
            "persist-me"
        );
        assert_eq!(settings["post_process_models"]["custom"], "llama3.2:3b");
        assert_eq!(
            settings["post_process_selected_prompt_id"],
            "handy_launcher_optimized"
        );
        assert_eq!(
            settings["post_process_prompts"][1]["id"],
            "handy_launcher_optimized"
        );
        assert_eq!(settings["post_process_prompts"][0]["id"], "existing");
    }

    #[test]
    fn merge_ollama_settings_updates_nested_settings_scope_when_present() {
        let mut root = json!({
            "settings": {
                "post_process_provider_id": "openai",
                "post_process_providers": {},
                "post_process_models": {},
                "post_process_prompts": {},
                "post_process_selected_prompt_id": "existing"
            },
            "window": {
                "theme": "dark"
            }
        });

        let settings = ConfigManager::settings_scope_mut(&mut root);
        ConfigManager::merge_ollama_settings(settings, "llama3.2:3b", 12400);

        assert_eq!(root["window"]["theme"], "dark");
        assert_eq!(root["settings"]["post_process_provider_id"], "custom");
        assert_eq!(
            root["settings"]["post_process_models"]["custom"],
            "llama3.2:3b"
        );
    }

    #[test]
    fn status_reads_from_nested_settings_scope() {
        let root = json!({
            "settings": {
                "post_process_provider_id": "custom",
                "post_process_models": {
                    "custom": "llama3.2:3b"
                },
                "post_process_selected_prompt_id": "handy_launcher_optimized"
            }
        });

        let settings = ConfigManager::settings_scope(&root);

        assert_eq!(settings["post_process_provider_id"], "custom");
        assert_eq!(settings["post_process_models"]["custom"], "llama3.2:3b");
        assert_eq!(
            settings["post_process_selected_prompt_id"],
            "handy_launcher_optimized"
        );
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandyConfig {
    #[serde(default = "default_provider_id")]
    pub post_process_provider_id: String,
    #[serde(default = "default_prompt_id")]
    pub post_process_selected_prompt_id: String,
    #[serde(default)]
    pub post_process_providers: BTreeMap<String, ProviderConfig>,
    #[serde(default)]
    pub post_process_models: BTreeMap<String, String>,
    #[serde(default)]
    pub post_process_prompts: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandyConfigStatus {
    pub config_path: Option<String>,
    pub latest_backup_path: Option<String>,
    pub config_exists: bool,
    pub handy_running: bool,
    pub current_provider_id: Option<String>,
    pub configured_model: Option<String>,
    pub selected_prompt_id: Option<String>,
}

fn default_provider_id() -> String {
    "ollama-local".into()
}

fn default_prompt_id() -> String {
    "default".into()
}

impl Default for HandyConfig {
    fn default() -> Self {
        Self {
            post_process_provider_id: default_provider_id(),
            post_process_selected_prompt_id: default_prompt_id(),
            post_process_providers: BTreeMap::new(),
            post_process_models: BTreeMap::new(),
            post_process_prompts: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HandyConfig;
    use serde_json::{json, Value};

    #[test]
    fn handy_config_round_trip_preserves_unknown_top_level_fields() {
        let original = json!({
            "post_process_provider_id": "custom-provider",
            "post_process_selected_prompt_id": "existing-prompt",
            "post_process_providers": {},
            "post_process_models": {},
            "post_process_prompts": {},
            "window_bounds": {
                "x": 140,
                "y": 220
            }
        });

        let parsed: HandyConfig =
            serde_json::from_value(original.clone()).expect("config should deserialize");
        let serialized = serde_json::to_value(parsed).expect("config should serialize");

        assert_eq!(serialized["window_bounds"], original["window_bounds"]);
    }

    #[test]
    fn handy_config_round_trip_preserves_unknown_provider_fields() {
        let original = json!({
            "post_process_provider_id": "custom-provider",
            "post_process_selected_prompt_id": "existing-prompt",
            "post_process_providers": {
                "custom-provider": {
                    "id": "custom-provider",
                    "model": "llama3",
                    "endpoint": "http://127.0.0.1:11434",
                    "headers": {
                        "Authorization": "Bearer local-token"
                    }
                }
            },
            "post_process_models": {},
            "post_process_prompts": {}
        });

        let parsed: HandyConfig =
            serde_json::from_value(original.clone()).expect("config should deserialize");
        let serialized = serde_json::to_value(parsed).expect("config should serialize");

        assert_eq!(
            serialized["post_process_providers"]["custom-provider"]["headers"],
            Value::Object(
                [(
                    "Authorization".to_string(),
                    Value::String("Bearer local-token".to_string())
                )]
                .into_iter()
                .collect()
            )
        );
    }
}

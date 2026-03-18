use tauri::command;

use crate::models::config::HandyConfigStatus;
use crate::services::config_manager::ConfigManager;
use serde_json::Value;

#[command]
pub fn get_handy_config() -> Result<Value, String> {
    ConfigManager::read_settings_json().map_err(|err| err.to_string())
}

#[command]
pub fn get_handy_config_status() -> Result<HandyConfigStatus, String> {
    ConfigManager::status().map_err(|err| err.to_string())
}

#[command]
pub fn configure_handy_with_ollama(model_name: String, port: u16) -> Result<Value, String> {
    ConfigManager::configure_handy_with_ollama(&model_name, port).map_err(|err| err.to_string())
}

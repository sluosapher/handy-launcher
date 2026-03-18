use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub total_ram_gb: f32,
    pub available_ram_gb: f32,
    pub total_disk_gb: f32,
    pub available_disk_gb: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugSnapshot {
    pub data_dir: Option<String>,
    pub log_path: Option<String>,
    pub recent_logs: Vec<String>,
}

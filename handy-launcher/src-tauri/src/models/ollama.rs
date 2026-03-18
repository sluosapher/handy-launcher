use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallProgress {
    pub percent: u8,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelDownloadProgress {
    pub model_name: String,
    pub progress: InstallProgress,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadedModel {
    pub name: String,
    pub size: u64,
    pub modified_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModelListResponse {
    pub models: Vec<DownloadedModel>,
}

#[derive(Debug, Deserialize)]
pub struct PullProgressEvent {
    pub status: String,
    pub completed: Option<u64>,
    pub total: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OllamaStatus {
    NotInstalled,
    Installing { progress: InstallProgress },
    Ready { version: String },
    Running { port: u16, version: Option<String> },
    Error { message: String },
}

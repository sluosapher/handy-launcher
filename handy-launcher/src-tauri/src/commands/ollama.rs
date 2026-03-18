use serde::Serialize;
use tauri::{command, AppHandle, Manager, Window};

use crate::models::ollama::{
    DownloadedModel, InstallProgress, ModelDownloadProgress, OllamaStatus,
};
use crate::services::ollama_manager::OllamaManager;
use crate::services::ollama_supervisor;
use crate::tray;

#[derive(Serialize)]
pub struct StartResult {
    port: u16,
    version: String,
}

#[command]
pub async fn check_ollama_status() -> Result<OllamaStatus, String> {
    Ok(OllamaManager::current_status().await)
}

#[command]
pub async fn install_ollama(app: AppHandle) -> Result<InstallProgress, String> {
    log::info!("install_ollama requested");
    let progress = OllamaManager::install_ollama()
        .await
        .map_err(|err| err.to_string())?;
    let status = OllamaManager::current_status().await;
    log::info!("install_ollama finished with status {:?}", status);
    tray::sync_tray_status(&app, &status);
    Ok(progress)
}

#[command]
pub fn ollama_binary_info() -> Result<String, String> {
    OllamaManager::binary_path()
        .map(|path| path.to_string_lossy().to_string())
        .ok_or_else(|| "Ollama binary not found".into())
}

#[command]
pub async fn start_ollama_server(
    app: AppHandle,
    port_hint: Option<u16>,
) -> Result<StartResult, String> {
    let port = port_hint
        .filter(|p| OllamaManager::port_range().contains(p))
        .unwrap_or_else(|| {
            OllamaManager::find_available_port().unwrap_or_else(OllamaManager::default_port)
        });
    log::info!("start_ollama_server requested on port {}", port);

    OllamaManager::start_server(port)
        .await
        .map_err(|err| err.to_string())?;

    let version = OllamaManager::wait_for_ready(port, OllamaManager::health_timeout_secs())
        .await
        .map_err(|err| err.to_string())?;

    let result = StartResult { port, version };
    let managed = app
        .state::<crate::models::app_state::AppState>()
        .mark_ollama_started(port);
    ollama_supervisor::spawn_managed_ollama_monitor(app.clone(), managed);
    let status = OllamaStatus::Running {
        port: result.port,
        version: Some(result.version.clone()),
    };
    log::info!(
        "Ollama running on port {} with version {}",
        result.port,
        result.version
    );
    tray::sync_tray_status(&app, &status);
    Ok(result)
}

#[command]
pub async fn verify_ollama_server(port: u16) -> Result<String, String> {
    OllamaManager::verify_server(port)
        .await
        .map_err(|err| err.to_string())
}

#[command]
pub async fn list_ollama_models(port_hint: Option<u16>) -> Result<Vec<DownloadedModel>, String> {
    let port = match port_hint.filter(|p| OllamaManager::port_range().contains(p)) {
        Some(port) => port,
        None => OllamaManager::discover_running()
            .await
            .map(|(port, _)| port)
            .ok_or_else(|| "Ollama is not running on any managed port".to_string())?,
    };

    OllamaManager::list_models(port)
        .await
        .map_err(|err| err.to_string())
}

#[command]
pub async fn download_ollama_model(
    window: Window,
    model_name: String,
    port_hint: Option<u16>,
) -> Result<InstallProgress, String> {
    let port = match port_hint.filter(|p| OllamaManager::port_range().contains(p)) {
        Some(port) => port,
        None => OllamaManager::discover_running()
            .await
            .map(|(port, _)| port)
            .ok_or_else(|| "Ollama is not running on any managed port".to_string())?,
    };

    OllamaManager::pull_model_with_progress(port, &model_name, |progress| {
        let _ = window.emit(
            "ollama-model-download-progress",
            ModelDownloadProgress {
                model_name: model_name.clone(),
                progress,
            },
        );
    })
    .await
    .map_err(|err| err.to_string())
}

#[command]
pub async fn stop_ollama_server(app: AppHandle) -> Result<usize, String> {
    log::info!("stop_ollama_server requested");
    app.state::<crate::models::app_state::AppState>()
        .clear_managed_ollama();
    let stopped = OllamaManager::stop_server().map_err(|err| err.to_string())?;
    let status = OllamaManager::current_status().await;
    log::info!("stop_ollama_server stopped {} process(es)", stopped);
    tray::sync_tray_status(&app, &status);
    Ok(stopped)
}

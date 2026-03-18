use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};

use crate::{
    models::{app_state::ManagedOllamaState, ollama::OllamaStatus},
    services::ollama_manager::OllamaManager,
    tray,
};

const MONITOR_INTERVAL_SECS: u64 = 3;
const RESTART_DELAY_SECS: u64 = 2;
const MAX_RESTARTS: u8 = 3;

pub fn spawn_managed_ollama_monitor<R: Runtime>(app: AppHandle<R>, managed: ManagedOllamaState) {
    tauri::async_runtime::spawn(async move {
        let Some(port) = managed.port else {
            return;
        };

        log::info!(
            "starting managed Ollama monitor for port {} generation {}",
            port,
            managed.generation
        );

        loop {
            tokio::time::sleep(Duration::from_secs(MONITOR_INTERVAL_SECS)).await;

            let state = app.state::<crate::models::app_state::AppState>();
            let current = state.managed_ollama();
            if current.port != Some(port) || current.generation != managed.generation {
                log::info!(
                    "stopping managed Ollama monitor for port {} generation {}",
                    port,
                    managed.generation
                );
                break;
            }

            if let Ok(version) = OllamaManager::verify_server(port).await {
                tray::sync_tray_status(
                    &app,
                    &OllamaStatus::Running {
                        port,
                        version: Some(version),
                    },
                );
                continue;
            }

            log::warn!(
                "managed Ollama on port {} became unhealthy; attempting restart",
                port
            );
            if !state.should_restart_ollama(port, managed.generation, MAX_RESTARTS) {
                log::error!(
                    "managed Ollama on port {} exceeded restart limit {}",
                    port,
                    MAX_RESTARTS
                );
                tray::sync_tray_status(
                    &app,
                    &OllamaStatus::Error {
                        message: "Ollama stopped and restart limit reached".into(),
                    },
                );
                break;
            }

            tokio::time::sleep(Duration::from_secs(RESTART_DELAY_SECS)).await;

            let restart_result = OllamaManager::start_server(port).await;
            match restart_result {
                Ok(()) => {
                    match OllamaManager::wait_for_ready(port, OllamaManager::health_timeout_secs())
                        .await
                    {
                        Ok(version) => {
                            log::info!(
                                "managed Ollama restart succeeded on port {} with version {}",
                                port,
                                version
                            );
                            state.mark_restart_succeeded(port, managed.generation);
                            tray::sync_tray_status(
                                &app,
                                &OllamaStatus::Running {
                                    port,
                                    version: Some(version),
                                },
                            );
                        }
                        Err(err) => {
                            log::error!(
                                "managed Ollama failed readiness check on port {}: {}",
                                port,
                                err
                            );
                            tray::sync_tray_status(
                                &app,
                                &OllamaStatus::Error {
                                    message: format!("Ollama restart failed: {err}"),
                                },
                            );
                        }
                    }
                }
                Err(err) => {
                    log::error!(
                        "managed Ollama restart command failed on port {}: {}",
                        port,
                        err
                    );
                    tray::sync_tray_status(
                        &app,
                        &OllamaStatus::Error {
                            message: format!("Ollama restart failed: {err}"),
                        },
                    );
                }
            }
        }
    });
}

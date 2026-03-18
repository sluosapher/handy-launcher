#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use handy_launcher::{
    commands::{config, ollama, system},
    models::app_state::AppState,
    services::ollama_manager::OllamaManager,
    tray,
    utils::logging,
};

fn main() {
    if let Err(err) = logging::init_logging() {
        eprintln!("failed to initialize logging: {err}");
    }
    let initial_status = tauri::async_runtime::block_on(OllamaManager::current_status());
    log::info!(
        "starting Handy Launcher with initial Ollama status: {:?}",
        initial_status
    );
    tauri::Builder::default()
        .manage(AppState::default())
        .system_tray(tray::build_system_tray(&initial_status))
        .on_system_tray_event(tray::handle_tray_event)
        .on_window_event(tray::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            ollama::check_ollama_status,
            ollama::install_ollama,
            ollama::ollama_binary_info,
            ollama::start_ollama_server,
            ollama::verify_ollama_server,
            ollama::list_ollama_models,
            ollama::download_ollama_model,
            ollama::stop_ollama_server,
            config::get_handy_config,
            config::get_handy_config_status,
            config::configure_handy_with_ollama,
            system::system_info,
            system::open_launcher_data_dir,
            system::open_ollama_download_page,
            system::open_handy_app,
            system::open_handy_download_page,
            system::get_launcher_debug_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running handy launcher");
}

use tauri::{
    AppHandle, CustomMenuItem, GlobalWindowEvent, Manager, Runtime, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};

use crate::models::ollama::OllamaStatus;
use crate::services::ollama_manager::OllamaManager;

pub const MENU_SHOW: &str = "show";
pub const MENU_STATUS: &str = "status";
pub const MENU_QUIT: &str = "quit";
pub const MAIN_WINDOW_LABEL: &str = "main";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayMenuAction {
    Show,
    Quit,
    Ignore,
}

pub fn build_system_tray(status: &OllamaStatus) -> SystemTray {
    let menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new(MENU_SHOW, "Show Handy Launcher"))
        .add_item(CustomMenuItem::new(MENU_STATUS, status_menu_title(status)).disabled())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new(MENU_QUIT, "Quit"));

    SystemTray::new().with_menu(menu)
}

pub fn menu_action(id: &str) -> TrayMenuAction {
    match id {
        MENU_SHOW => TrayMenuAction::Show,
        MENU_QUIT => TrayMenuAction::Quit,
        _ => TrayMenuAction::Ignore,
    }
}

pub fn status_menu_title(status: &OllamaStatus) -> String {
    match status {
        OllamaStatus::Running { .. } => "Ollama: Running".into(),
        OllamaStatus::Ready { .. } => "Ollama: Ready".into(),
        OllamaStatus::Installing { .. } => "Ollama: Installing".into(),
        OllamaStatus::NotInstalled => "Ollama: Not installed".into(),
        OllamaStatus::Error { .. } => "Ollama: Error".into(),
    }
}

pub fn sync_tray_status<R: Runtime>(app: &AppHandle<R>, status: &OllamaStatus) {
    let _ = app
        .tray_handle()
        .get_item(MENU_STATUS)
        .set_title(status_menu_title(status));
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_window(MAIN_WINDOW_LABEL) {
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

pub fn toggle_main_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_window(MAIN_WINDOW_LABEL) {
        if window.is_visible()? {
            window.hide()?;
        } else {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}

pub fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            let _ = toggle_main_window(app);
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match menu_action(&id) {
            TrayMenuAction::Show => {
                let _ = show_main_window(app);
            }
            TrayMenuAction::Quit => {
                app.state::<crate::models::app_state::AppState>()
                    .clear_managed_ollama();
                let _ = OllamaManager::stop_server();
                app.exit(0);
            }
            TrayMenuAction::Ignore => {}
        },
        _ => {}
    }
}

pub fn handle_window_event<R: Runtime>(event: GlobalWindowEvent<R>) {
    if event.window().label() != MAIN_WINDOW_LABEL {
        return;
    }

    if let WindowEvent::CloseRequested { api, .. } = event.event() {
        api.prevent_close();
        let _ = event.window().hide();
    }
}

#[cfg(test)]
mod tests {
    use super::{menu_action, status_menu_title, TrayMenuAction};
    use crate::models::ollama::{InstallProgress, OllamaStatus};

    #[test]
    fn menu_action_maps_known_ids() {
        assert_eq!(menu_action("show"), TrayMenuAction::Show);
        assert_eq!(menu_action("quit"), TrayMenuAction::Quit);
        assert_eq!(menu_action("status"), TrayMenuAction::Ignore);
        assert_eq!(menu_action("unknown"), TrayMenuAction::Ignore);
    }

    #[test]
    fn status_menu_title_reflects_ollama_state() {
        assert_eq!(
            status_menu_title(&OllamaStatus::NotInstalled),
            "Ollama: Not installed"
        );
        assert_eq!(
            status_menu_title(&OllamaStatus::Installing {
                progress: InstallProgress {
                    percent: 42,
                    status: "downloading".into(),
                },
            }),
            "Ollama: Installing"
        );
        assert_eq!(
            status_menu_title(&OllamaStatus::Ready {
                version: "0.7.0".into(),
            }),
            "Ollama: Ready"
        );
        assert_eq!(
            status_menu_title(&OllamaStatus::Running {
                port: 11434,
                version: Some("0.7.0".into()),
            }),
            "Ollama: Running"
        );
        assert_eq!(
            status_menu_title(&OllamaStatus::Error {
                message: "boom".into(),
            }),
            "Ollama: Error"
        );
    }
}

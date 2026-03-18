use crate::models::system::{DebugSnapshot, SystemInfo};
use crate::utils::{
    logging,
    paths::{handy_app_path, launcher_data_dir, launcher_log_path},
};
use sysinfo::{Disks, System};
use tauri::api::shell;
use tauri::{command, AppHandle, Manager};

#[command]
pub fn system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();
    let disks = Disks::new_with_refreshed_list();

    let total_ram = sys.total_memory();
    let available_ram = sys.available_memory();

    let (total_disk, available_disk) = disks
        .list()
        .iter()
        .fold((0_u64, 0_u64), |(total, avail), disk| {
            (total + disk.total_space(), avail + disk.available_space())
        });

    let to_gb = |kilobytes: u64| -> f32 { (kilobytes as f32) / 1024.0 / 1024.0 };

    SystemInfo {
        os_name: System::name().unwrap_or_else(|| "Unknown".into()),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".into()),
        total_ram_gb: to_gb(total_ram),
        available_ram_gb: to_gb(available_ram),
        total_disk_gb: (total_disk as f32) / 1_073_741_824.0,
        available_disk_gb: (available_disk as f32) / 1_073_741_824.0,
    }
}

#[command]
pub fn open_launcher_data_dir(app: AppHandle) -> Result<(), String> {
    let target =
        launcher_data_dir().ok_or_else(|| "Launcher data directory unavailable".to_string())?;
    shell::open(&app.shell_scope(), target.display().to_string(), None)
        .map_err(|err| err.to_string())
}

#[command]
pub fn open_handy_app(app: AppHandle) -> Result<(), String> {
    let target = handy_app_path()
        .ok_or_else(|| "Handy app not found. Download it from https://pa.is/handy".to_string())?;
    shell::open(&app.shell_scope(), target.display().to_string(), None)
        .map_err(|err| err.to_string())
}

#[command]
pub fn open_ollama_download_page(app: AppHandle) -> Result<(), String> {
    shell::open(&app.shell_scope(), "https://ollama.com/download", None)
        .map_err(|err| err.to_string())
}

#[command]
pub fn open_handy_download_page(app: AppHandle) -> Result<(), String> {
    shell::open(&app.shell_scope(), "https://pa.is/handy", None).map_err(|err| err.to_string())
}

#[command]
pub fn get_launcher_debug_snapshot(line_limit: Option<usize>) -> Result<DebugSnapshot, String> {
    let limit = line_limit.unwrap_or(50);
    Ok(DebugSnapshot {
        data_dir: launcher_data_dir().map(|path| path.display().to_string()),
        log_path: launcher_log_path().map(|path| path.display().to_string()),
        recent_logs: logging::read_log_tail(limit).map_err(|err| err.to_string())?,
    })
}

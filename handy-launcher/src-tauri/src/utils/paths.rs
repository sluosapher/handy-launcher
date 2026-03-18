use std::{env, fs, path::PathBuf};

fn handy_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var("APPDATA")
            .ok()
            .map(PathBuf::from)
            .map(|p| p.join("com.pais.handy"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| home.join("Library/Application Support/com.pais.handy"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        dirs::data_dir().map(|p| p.join("com.pais.handy"))
    }
}

pub fn handy_config_path() -> Option<PathBuf> {
    handy_data_dir().map(|base| base.join("settings_store.json"))
}

pub fn handy_config_backup_path() -> Option<PathBuf> {
    handy_data_dir().map(|base| base.join("settings_store.json.bak"))
}

pub fn launcher_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|base| base.join("HandyLauncher"))
}

pub fn launcher_logs_dir_from(base: &std::path::Path) -> PathBuf {
    base.join("logs")
}

pub fn launcher_log_path_from(base: &std::path::Path) -> PathBuf {
    launcher_logs_dir_from(base).join("handy-launcher.log")
}

pub fn launcher_logs_dir() -> Option<PathBuf> {
    launcher_data_dir().map(|base| launcher_logs_dir_from(&base))
}

pub fn launcher_log_path() -> Option<PathBuf> {
    launcher_data_dir().map(|base| launcher_log_path_from(&base))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{launcher_log_path_from, launcher_logs_dir_from};

    #[test]
    fn launcher_logs_dir_lives_under_launcher_data_dir() {
        let logs_dir =
            launcher_logs_dir_from(Path::new("C:/Users/test/AppData/Roaming/HandyLauncher"));

        assert_eq!(
            logs_dir,
            Path::new("C:/Users/test/AppData/Roaming/HandyLauncher").join("logs")
        );
    }

    #[test]
    fn launcher_log_path_uses_default_log_file_name() {
        let log_path =
            launcher_log_path_from(Path::new("C:/Users/test/AppData/Roaming/HandyLauncher"));

        assert_eq!(
            log_path,
            Path::new("C:/Users/test/AppData/Roaming/HandyLauncher")
                .join("logs")
                .join("handy-launcher.log")
        );
    }
}

pub fn handy_app_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();

        if let Ok(program_files) = env::var("ProgramFiles") {
            candidates.push(PathBuf::from(program_files).join("Handy").join("Handy.exe"));
        }

        if let Some(local_data) = dirs::data_local_dir() {
            candidates.push(local_data.join("Handy").join("Handy.exe"));
            candidates.push(local_data.join("Programs").join("Handy").join("Handy.exe"));
        }

        return candidates.into_iter().find(|candidate| candidate.exists());
    }

    #[cfg(target_os = "macos")]
    {
        let system_app = PathBuf::from("/Applications/Handy.app");
        if system_app.exists() {
            return Some(system_app);
        }

        return dirs::home_dir()
            .map(|home| home.join("Applications/Handy.app"))
            .filter(|candidate| candidate.exists());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

pub fn handy_backup_dir() -> Option<PathBuf> {
    launcher_data_dir().map(|base| base.join("backups"))
}

pub fn new_handy_config_backup_path() -> Option<PathBuf> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    handy_backup_dir().map(|base| base.join(format!("settings_backup_{timestamp}.json")))
}

pub fn latest_handy_config_backup_path() -> Option<PathBuf> {
    let backup_dir = handy_backup_dir()?;
    let mut backups: Vec<PathBuf> = fs::read_dir(backup_dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("settings_backup_") && name.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect();

    backups.sort();
    backups.pop()
}

pub fn ollama_data_dir() -> Option<PathBuf> {
    launcher_data_dir().map(|base| base.join("ollama"))
}

pub fn ollama_download_dir() -> Option<PathBuf> {
    ollama_data_dir().map(|base| base.join("downloads"))
}

pub fn ollama_installer_path(file_name: &str) -> Option<PathBuf> {
    ollama_download_dir().map(|base| base.join(file_name))
}

pub fn ollama_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "ollama.exe"
    } else {
        "ollama"
    }
}

pub fn ollama_binary_path() -> Option<PathBuf> {
    if let Some(dir) = ollama_data_dir() {
        let candidate = dir.join(ollama_binary_name());
        if candidate.exists() {
            return Some(candidate);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_bundle_paths = [
            dirs::home_dir()
                .map(|home| home.join("Applications/Ollama.app/Contents/Resources/ollama")),
            Some(PathBuf::from(
                "/Applications/Ollama.app/Contents/Resources/ollama",
            )),
        ];

        for candidate in app_bundle_paths.into_iter().flatten() {
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|p| p.join(ollama_binary_name()))
            .find(|candidate| candidate.exists())
    })
}

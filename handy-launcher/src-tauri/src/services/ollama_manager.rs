use std::{
    fs, io,
    net::TcpListener,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessesToUpdate, System};
use tokio::process::Command;

use crate::models::ollama::{
    DownloadedModel, InstallProgress, ModelListResponse, OllamaStatus, PullProgressEvent,
};
use crate::utils::http::RetryingHttpClient;
use crate::utils::paths::{
    ollama_binary_name, ollama_binary_path, ollama_data_dir, ollama_download_dir,
    ollama_installer_path,
};

const OLLAMA_PORT_START: u16 = 63452;
const OLLAMA_PORT_END: u16 = 63462;
const HEALTH_TIMEOUT_SECS: u64 = 30;
const HEALTH_POLL_INTERVAL_MS: u64 = 500;
const SERVER_HOST: &str = "127.0.0.1";
const WINDOWS_INSTALLER_URL: &str = "https://ollama.com/download/OllamaSetup.exe";
const MACOS_INSTALLER_URL: &str = "https://ollama.com/download/Ollama.dmg";

#[derive(Debug, thiserror::Error)]
pub enum OllamaError {
    #[error("Ollama binary not found")]
    BinaryNotFound,
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Server did not respond within {0} seconds")]
    Timeout(u64),
    #[error("Unexpected server response: {0}")]
    UnexpectedStatus(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("Installer not found at {0}")]
    InstallerNotFound(String),
    #[error("Installation failed: {0}")]
    InstallFailed(String),
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: Option<String>,
}

#[derive(Debug, Serialize)]
struct PullModelRequest<'a> {
    name: &'a str,
    stream: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstallerArtifact {
    file_name: &'static str,
    download_url: &'static str,
}

pub struct OllamaManager;

impl OllamaManager {
    pub fn data_dir() -> Option<PathBuf> {
        if let Some(dir) = ollama_data_dir() {
            if !dir.exists() {
                let _ = fs::create_dir_all(&dir);
            }
            Some(dir)
        } else {
            None
        }
    }

    pub fn ensure_data_dir() -> io::Result<()> {
        if let Some(dir) = Self::data_dir() {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn ensure_download_dir() -> io::Result<PathBuf> {
        let dir = ollama_download_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "download directory unavailable")
        })?;
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn binary_path() -> Option<PathBuf> {
        ollama_binary_path()
    }

    pub fn binary_name() -> &'static str {
        ollama_binary_name()
    }

    pub fn is_installed() -> bool {
        Self::binary_path().is_some()
    }

    pub async fn version() -> Option<String> {
        let binary = Self::binary_path()?;
        let version_commands = [["--version"], ["-v"], ["version"]];

        for args in version_commands {
            let output = Command::new(&binary).args(args).output().await.ok()?;
            if !output.status.success() {
                continue;
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = if stderr.trim().is_empty() {
                stdout.to_string()
            } else if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                format!("{stdout}\n{stderr}")
            };

            if let Some(version) = Self::parse_version_output(&combined) {
                return Some(version);
            }
        }

        None
    }

    pub async fn current_status() -> OllamaStatus {
        if !Self::is_installed() {
            return OllamaStatus::NotInstalled;
        }

        if let Some((port, version)) = Self::discover_running().await {
            return OllamaStatus::Running {
                port,
                version: Some(version),
            };
        }

        match Self::version().await {
            Some(version) => OllamaStatus::Ready { version },
            None => OllamaStatus::Error {
                message: "installed but version lookup failed".into(),
            },
        }
    }

    pub fn install_progress_placeholder() -> InstallProgress {
        InstallProgress {
            percent: 0,
            status: "not started".into(),
        }
    }

    pub fn port_range() -> std::ops::RangeInclusive<u16> {
        OLLAMA_PORT_START..=OLLAMA_PORT_END
    }

    pub fn default_port() -> u16 {
        OLLAMA_PORT_START
    }

    pub fn health_timeout_secs() -> u64 {
        HEALTH_TIMEOUT_SECS
    }

    pub fn find_available_port() -> Option<u16> {
        for port in Self::port_range() {
            if TcpListener::bind((SERVER_HOST, port)).is_ok() {
                return Some(port);
            }
        }
        None
    }

    pub async fn start_server(port: u16) -> Result<(), OllamaError> {
        let binary = Self::binary_path().ok_or(OllamaError::BinaryNotFound)?;
        Self::ensure_data_dir()?;

        let mut child = Command::new(binary)
            .arg("serve")
            .arg("--host")
            .arg(SERVER_HOST)
            .arg("--port")
            .arg(port.to_string())
            .spawn()?;

        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        Ok(())
    }

    pub async fn verify_server(port: u16) -> Result<String, OllamaError> {
        let client = Self::http_client()?;
        let url = format!("http://{}:{}/api/version", SERVER_HOST, port);
        let response = client
            .get_json::<VersionResponse>(&url)
            .await
            .map_err(OllamaError::UnexpectedStatus)?;
        Ok(response.version.unwrap_or_else(|| "unknown".into()))
    }

    pub async fn discover_running() -> Option<(u16, String)> {
        for port in Self::port_range() {
            if let Ok(version) = Self::verify_server(port).await {
                return Some((port, version));
            }
        }
        None
    }

    pub async fn list_models(port: u16) -> Result<Vec<DownloadedModel>, OllamaError> {
        let client = Self::http_client()?;
        let url = format!("http://{}:{}/api/tags", SERVER_HOST, port);
        let body = client
            .get_text(&url)
            .await
            .map_err(OllamaError::UnexpectedStatus)?;
        Self::parse_models_response(&body)
    }

    pub async fn pull_model(port: u16, model_name: &str) -> Result<InstallProgress, OllamaError> {
        Self::pull_model_with_progress(port, model_name, |_| {}).await
    }

    pub async fn pull_model_with_progress<F>(
        port: u16,
        model_name: &str,
        mut on_progress: F,
    ) -> Result<InstallProgress, OllamaError>
    where
        F: FnMut(InstallProgress),
    {
        let client = Self::http_client()?;
        let url = format!("http://{}:{}/api/pull", SERVER_HOST, port);
        let response = client
            .post_json(
                &url,
                &PullModelRequest {
                    name: model_name,
                    stream: true,
                },
            )
            .await
            .map_err(OllamaError::UnexpectedStatus)?;

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut latest = InstallProgress {
            percent: 0,
            status: "starting download".into(),
        };

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(index) = buffer.find('\n') {
                let line = buffer[..index].trim().to_string();
                buffer = buffer[index + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                latest = Self::parse_pull_progress_line(&line)?;
                on_progress(latest.clone());
                if latest.status == "success" {
                    return Ok(latest);
                }
            }
        }

        if !buffer.trim().is_empty() {
            latest = Self::parse_pull_progress_line(buffer.trim())?;
            on_progress(latest.clone());
        }

        Ok(latest)
    }

    pub async fn install_ollama() -> Result<InstallProgress, OllamaError> {
        if Self::is_installed() {
            let version = Self::version().await.unwrap_or_else(|| "unknown".into());
            return Ok(InstallProgress {
                percent: 100,
                status: format!("already installed ({version})"),
            });
        }

        let artifact = Self::current_installer_artifact()?;
        let installer_path = Self::download_installer(&artifact).await?;
        Self::install_downloaded_artifact(&installer_path).await?;

        if !Self::is_installed() {
            return Err(OllamaError::InstallFailed(
                "installer completed but Ollama binary is still missing".into(),
            ));
        }

        Ok(InstallProgress {
            percent: 100,
            status: "installation complete".into(),
        })
    }

    pub async fn wait_for_ready(port: u16, timeout_secs: u64) -> Result<String, OllamaError> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            match Self::verify_server(port).await {
                Ok(version) => return Ok(version),
                Err(_) => tokio::time::sleep(Duration::from_millis(HEALTH_POLL_INTERVAL_MS)).await,
            }
        }

        Err(OllamaError::Timeout(timeout_secs))
    }

    pub fn stop_server() -> Result<usize, OllamaError> {
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut stopped = 0;
        for process in sys.processes().values() {
            if process
                .name()
                .to_string_lossy()
                .to_lowercase()
                .contains("ollama")
            {
                if process.kill() {
                    stopped += 1;
                }
            }
        }

        Ok(stopped)
    }

    fn http_client() -> Result<RetryingHttpClient, OllamaError> {
        RetryingHttpClient::new(Duration::from_secs(5), 3).map_err(OllamaError::Http)
    }

    fn current_installer_artifact() -> Result<InstallerArtifact, OllamaError> {
        Self::installer_artifact_for(std::env::consts::OS, std::env::consts::ARCH)
    }

    fn installer_artifact_for(os: &str, _arch: &str) -> Result<InstallerArtifact, OllamaError> {
        match os {
            "windows" => Ok(InstallerArtifact {
                file_name: "OllamaSetup.exe",
                download_url: WINDOWS_INSTALLER_URL,
            }),
            "macos" => Ok(InstallerArtifact {
                file_name: "Ollama.dmg",
                download_url: MACOS_INSTALLER_URL,
            }),
            unsupported => Err(OllamaError::UnsupportedPlatform(unsupported.into())),
        }
    }

    async fn download_installer(artifact: &InstallerArtifact) -> Result<PathBuf, OllamaError> {
        Self::ensure_download_dir()?;

        let client = Self::http_client()?;
        let bytes = client
            .get_bytes(artifact.download_url)
            .await
            .map_err(OllamaError::UnexpectedStatus)?;
        let installer_path = ollama_installer_path(artifact.file_name)
            .ok_or_else(|| OllamaError::InstallFailed("installer path unavailable".into()))?;

        fs::write(&installer_path, bytes)?;
        Ok(installer_path)
    }

    async fn install_downloaded_artifact(installer_path: &Path) -> Result<(), OllamaError> {
        if !installer_path.exists() {
            return Err(OllamaError::InstallerNotFound(
                installer_path.display().to_string(),
            ));
        }

        #[cfg(target_os = "windows")]
        {
            let install_dir = Self::data_dir().ok_or_else(|| {
                OllamaError::InstallFailed("install directory unavailable".into())
            })?;
            fs::create_dir_all(&install_dir)?;

            let status = Command::new(installer_path)
                .args(["/SP-", "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"])
                .arg(format!("/DIR={}", install_dir.display()))
                .status()
                .await?;

            if status.success() {
                return Ok(());
            }

            return Err(OllamaError::InstallFailed(format!(
                "installer exited with status {status}"
            )));
        }

        #[cfg(target_os = "macos")]
        {
            let home = dirs::home_dir()
                .ok_or_else(|| OllamaError::InstallFailed("home directory unavailable".into()))?;
            let applications_dir = home.join("Applications");
            fs::create_dir_all(&applications_dir)?;

            let mount_point = std::env::temp_dir().join("handy-launcher-ollama-mount");
            if mount_point.exists() {
                let _ = fs::remove_dir_all(&mount_point);
            }
            fs::create_dir_all(&mount_point)?;

            let attach = Command::new("hdiutil")
                .args([
                    "attach",
                    installer_path.to_str().ok_or_else(|| {
                        OllamaError::InstallFailed("invalid installer path".into())
                    })?,
                    "-nobrowse",
                    "-readonly",
                    "-mountpoint",
                    mount_point
                        .to_str()
                        .ok_or_else(|| OllamaError::InstallFailed("invalid mount point".into()))?,
                ])
                .status()
                .await?;

            if !attach.success() {
                return Err(OllamaError::InstallFailed(format!(
                    "failed to mount dmg: {attach}"
                )));
            }

            let app_source = mount_point.join("Ollama.app");
            let app_target = applications_dir.join("Ollama.app");
            if app_target.exists() {
                fs::remove_dir_all(&app_target)?;
            }

            let copy_status = Command::new("cp")
                .args([
                    "-R",
                    app_source.to_str().ok_or_else(|| {
                        OllamaError::InstallFailed("invalid app source path".into())
                    })?,
                    app_target.to_str().ok_or_else(|| {
                        OllamaError::InstallFailed("invalid app target path".into())
                    })?,
                ])
                .status()
                .await?;

            let detach_status = Command::new("hdiutil")
                .args([
                    "detach",
                    mount_point
                        .to_str()
                        .ok_or_else(|| OllamaError::InstallFailed("invalid mount point".into()))?,
                ])
                .status()
                .await?;

            let _ = fs::remove_dir_all(&mount_point);

            if !copy_status.success() {
                return Err(OllamaError::InstallFailed(format!(
                    "failed to copy app bundle: {copy_status}"
                )));
            }

            if !detach_status.success() {
                return Err(OllamaError::InstallFailed(format!(
                    "failed to detach dmg: {detach_status}"
                )));
            }

            return Ok(());
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = installer_path;
            Err(OllamaError::UnsupportedPlatform(
                std::env::consts::OS.into(),
            ))
        }
    }

    fn parse_models_response(body: &str) -> Result<Vec<DownloadedModel>, OllamaError> {
        serde_json::from_str::<ModelListResponse>(body)
            .map(|response| response.models)
            .map_err(|err| OllamaError::UnexpectedStatus(err.to_string()))
    }

    fn parse_version_output(output: &str) -> Option<String> {
        output
            .lines()
            .filter(|line| line.to_ascii_lowercase().contains("version"))
            .flat_map(|line| line.split_whitespace())
            .map(|token| token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.'))
            .find(|token| Self::looks_like_version_token(token))
            .map(str::to_string)
    }

    fn looks_like_version_token(token: &str) -> bool {
        token.chars().any(|ch| ch.is_ascii_digit())
            && token.contains('.')
            && token
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-')
    }

    fn parse_pull_progress_line(line: &str) -> Result<InstallProgress, OllamaError> {
        let event = serde_json::from_str::<PullProgressEvent>(line)
            .map_err(|err| OllamaError::UnexpectedStatus(err.to_string()))?;

        if event.status == "success" {
            return Ok(InstallProgress {
                percent: 100,
                status: event.status,
            });
        }

        let percent = match (event.completed, event.total) {
            (Some(completed), Some(total)) if total > 0 => ((completed as f64 / total as f64)
                * 100.0)
                .round()
                .clamp(0.0, 100.0) as u8,
            _ => 0,
        };

        Ok(InstallProgress {
            percent,
            status: event.status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_output_extracts_version_from_warning_style_output() {
        let version = OllamaManager::parse_version_output(
            "Warning: could not connect to a running Ollama instance\nWarning: client version is 0.16.1\n",
        );

        assert_eq!(version.as_deref(), Some("0.16.1"));
    }

    #[test]
    fn parse_version_output_returns_none_when_no_version_token_exists() {
        let version = OllamaManager::parse_version_output("Warning: could not connect\n");

        assert!(version.is_none());
    }

    #[test]
    fn parse_pull_progress_line_computes_percent_from_completed_bytes() {
        let progress = OllamaManager::parse_pull_progress_line(
            r#"{"status":"downloading","completed":50,"total":200}"#,
        )
        .expect("progress should parse");

        assert_eq!(progress.percent, 25);
        assert_eq!(progress.status, "downloading");
    }

    #[test]
    fn parse_pull_progress_line_marks_success_as_complete() {
        let progress = OllamaManager::parse_pull_progress_line(r#"{"status":"success"}"#)
            .expect("success payload should parse");

        assert_eq!(progress.percent, 100);
        assert_eq!(progress.status, "success");
    }

    #[test]
    fn parse_models_response_extracts_downloaded_models() {
        let models = OllamaManager::parse_models_response(
            r#"{
                "models": [
                    {
                        "name": "llama3.2:1b",
                        "size": 1300000000,
                        "modified_at": "2026-03-17T10:30:00Z"
                    }
                ]
            }"#,
        )
        .expect("model list should parse");

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "llama3.2:1b");
        assert_eq!(models[0].size, 1_300_000_000);
    }

    #[test]
    fn installer_artifact_for_windows_matches_official_download() {
        let artifact = OllamaManager::installer_artifact_for("windows", "x86_64")
            .expect("windows artifact should resolve");

        assert_eq!(artifact.file_name, "OllamaSetup.exe");
        assert_eq!(artifact.download_url, WINDOWS_INSTALLER_URL);
    }

    #[test]
    fn installer_artifact_for_macos_matches_official_download() {
        let artifact = OllamaManager::installer_artifact_for("macos", "aarch64")
            .expect("mac artifact should resolve");

        assert_eq!(artifact.file_name, "Ollama.dmg");
        assert_eq!(artifact.download_url, MACOS_INSTALLER_URL);
    }
}

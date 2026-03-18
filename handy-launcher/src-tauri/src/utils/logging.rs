use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use log::LevelFilter;

use crate::utils::paths::launcher_log_path;

pub const MAX_LOG_SIZE_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_LOG_FILES: usize = 5;

static LOGGER_READY: OnceLock<()> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("launcher log path unavailable")]
    MissingLogPath,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Init(#[from] log::SetLoggerError),
}

pub fn init_logging() -> Result<PathBuf, LoggingError> {
    if LOGGER_READY.get().is_some() {
        return launcher_log_path().ok_or(LoggingError::MissingLogPath);
    }

    let log_path = launcher_log_path().ok_or(LoggingError::MissingLogPath)?;
    let log_dir = log_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "log directory unavailable"))?;

    fs::create_dir_all(log_dir)?;
    rotate_log_files(&log_path, MAX_LOG_SIZE_BYTES, MAX_LOG_FILES)?;

    fern::Dispatch::new()
        .level(default_log_level())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(fern::log_file(&log_path)?)
        .apply()?;

    let _ = LOGGER_READY.set(());
    log::info!("logging initialized at {}", log_path.display());
    Ok(log_path)
}

fn default_log_level() -> LevelFilter {
    match std::env::var("RUST_LOG")
        .ok()
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("trace") => LevelFilter::Trace,
        Some("debug") => LevelFilter::Debug,
        Some("warn") => LevelFilter::Warn,
        Some("error") => LevelFilter::Error,
        Some("off") => LevelFilter::Off,
        _ => LevelFilter::Info,
    }
}

pub fn rotate_log_files(active_log: &Path, max_bytes: u64, max_files: usize) -> io::Result<()> {
    if !active_log.exists() || max_files == 0 {
        return Ok(());
    }

    if fs::metadata(active_log)?.len() <= max_bytes {
        return Ok(());
    }

    for index in (1..=max_files).rev() {
        let destination = archived_log_path(active_log, index);
        if destination.exists() {
            fs::remove_file(&destination)?;
        }

        let source = if index == 1 {
            active_log.to_path_buf()
        } else {
            archived_log_path(active_log, index - 1)
        };

        if source.exists() {
            fs::rename(source, destination)?;
        }
    }

    Ok(())
}

pub fn read_log_tail(line_limit: usize) -> Result<Vec<String>, LoggingError> {
    let log_path = launcher_log_path().ok_or(LoggingError::MissingLogPath)?;
    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(log_path)?;
    Ok(tail_log_lines(&contents, line_limit))
}

pub fn tail_log_lines(contents: &str, line_limit: usize) -> Vec<String> {
    if line_limit == 0 {
        return Vec::new();
    }

    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .rev()
        .take(line_limit)
        .map(str::to_string)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn archived_log_path(active_log: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.{}", active_log.display(), index))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{rotate_log_files, tail_log_lines, MAX_LOG_FILES, MAX_LOG_SIZE_BYTES};

    fn temp_logs_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "handy-launcher-logging-tests-{test_name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn rotate_log_files_promotes_existing_archives_and_caps_file_count() {
        let dir = temp_logs_dir("rotate");
        let active = dir.join("handy-launcher.log");

        fs::write(&active, vec![b'a'; (MAX_LOG_SIZE_BYTES + 1) as usize]).expect("active log");
        fs::write(dir.join("handy-launcher.log.1"), b"1").expect("archive 1");
        fs::write(dir.join("handy-launcher.log.2"), b"2").expect("archive 2");
        fs::write(
            dir.join(format!("handy-launcher.log.{}", MAX_LOG_FILES)),
            b"drop",
        )
        .expect("max archive");

        rotate_log_files(&active, MAX_LOG_SIZE_BYTES, MAX_LOG_FILES).expect("rotation should work");

        assert!(!active.exists());
        assert!(dir.join("handy-launcher.log.1").exists());
        assert!(dir.join("handy-launcher.log.2").exists());
        assert!(dir.join("handy-launcher.log.3").exists());
        assert!(!dir
            .join(format!("handy-launcher.log.{}", MAX_LOG_FILES + 1))
            .exists());
    }

    #[test]
    fn rotate_log_files_skips_rotation_when_log_is_under_limit() {
        let dir = temp_logs_dir("skip");
        let active = dir.join("handy-launcher.log");

        fs::write(&active, vec![b'a'; 1024]).expect("active log");

        rotate_log_files(&active, MAX_LOG_SIZE_BYTES, MAX_LOG_FILES).expect("rotation should work");

        assert!(active.exists());
        assert!(!dir.join("handy-launcher.log.1").exists());
    }

    #[test]
    fn tail_log_lines_returns_the_latest_lines_in_order() {
        let tail = tail_log_lines("one\ntwo\nthree\nfour\n", 2);

        assert_eq!(tail, vec!["three".to_string(), "four".to_string()]);
    }

    #[test]
    fn tail_log_lines_skips_empty_lines_at_the_end() {
        let tail = tail_log_lines("one\ntwo\n\n", 2);

        assert_eq!(tail, vec!["one".to_string(), "two".to_string()]);
    }
}

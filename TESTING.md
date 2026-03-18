# Handy Launcher Testing

## Current Verification Commands

Run these from the nested app unless noted otherwise.

### Frontend

```powershell
cd handy-launcher
bun run check
bun test
```

### Rust Backend

```powershell
cd handy-launcher/src-tauri
cargo test --lib
cargo check
```

### Focused Rust Tests

```powershell
cd handy-launcher/src-tauri
cargo test launcher_logs_dir_lives_under_launcher_data_dir --lib
cargo test rotate_log_files_promotes_existing_archives_and_caps_file_count --lib
```

## Logging

- Active log file: launcher data directory `logs/handy-launcher.log`
- Rotation policy: rotate when the active file exceeds 10 MB
- Retention: keep the latest 5 archived log files

## Manual Smoke Checklist

- Start the launcher and confirm `handy-launcher.log` is created under the launcher data directory.
- Start Ollama from the launcher and confirm the log records the selected port and version.
- Stop Ollama from the launcher and confirm the stop request and stopped-process count are logged.
- Close the window and confirm the app remains available in the tray.
- If possible, terminate a launcher-managed Ollama process externally and confirm the log records the restart attempt.
- On the status dashboard, click the `Handy launcher` label 5 times within 5 seconds and confirm the hidden debug panel appears.
- In the debug panel, confirm the latest log lines, current Ollama state, and Handy config status are visible.

## Notes

- `bun test` may print an `EPERM` line referencing `C:\Users\sluo_\`; in this workspace it has still exited 0 when assertions pass.
- Tauri runtime behaviors such as tray interactions and live restart recovery still need manual validation in addition to the automated checks above.

# Handy Launcher - Debugging Guide

**When you encounter errors during implementation, refer to this guide.**

---

## Quick Diagnostic Commands

### Check Environment
```bash
# Verify Rust
rustc --version
cargo --version

# Verify Node/Bun
node --version
bun --version

# Verify Tauri CLI
bun run tauri --version
```

### Check Ollama Status
```bash
# Test if Ollama is running
curl http://127.0.0.1:63452/api/tags

# Check Ollama version
curl http://127.0.0.1:63452/api/version

# Test model generation
curl -X POST http://127.0.0.1:63452/api/generate -d '{"model":"llama3.2:1b","prompt":"Hello"}'
```

### Check Handy Config Location
```powershell
# Windows
$env:APPDATA
Test-Path "$env:APPDATA/com.pais.handy/settings_store.json"
Get-Content "$env:APPDATA/com.pais.handy/settings_store.json" | ConvertFrom-Json

# macOS
ls ~/Library/Application\ Support/com.pais.handy/
cat ~/Library/Application\ Support/com.pais.handy/settings_store.json
```

---

## Common Errors by Phase

### Phase 0: Scaffold Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `create-tauri-app` not found | Bun not in PATH | Add Bun to PATH, restart shell |
| `error: linker link.exe not found` | Missing MSVC | Install Visual Studio Build Tools |
| `Failed to run cargo` | Rust not installed | Run `rustup default stable` |
| `tauri.conf.json` parse error | JSON syntax | Validate JSON at jsonlint.com |
| `Cannot find module '@tauri-apps/api'` | Missing install | Run `bun install` |

### Phase 1: Ollama Manager Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Connection refused` | Ollama not started | Check if process spawned, check port |
| `Permission denied` | Binary not executable | Run `chmod +x ollama` on Unix |
| `Address already in use` | Port 63452 taken | Use `find_available_port()` |
| `Download failed` | Network issue | Add retry with exponential backoff |
| `Process exited immediately` | Ollama error | Check Ollama logs at `~/.ollama/logs` |
| `API timeout` | Ollama slow | Increase timeout to 60s |

#### Debug Ollama Process
```rust
// Add verbose logging to spawn
let output = std::process::Command::new(&ollama_path)
    .arg("serve")
    .env("OLLAMA_HOST", format!("127.0.0.1:{}", port))
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn();

// Log stdout/stderr
```

### Phase 2: Config Manager Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `File not found` | Handy not installed | Create config dir manually |
| `Permission denied` | File locked by Handy | Close Handy app first |
| `Invalid JSON` | Corrupted settings | Backup and recreate |
| `Key not found` | Missing field | Use `.get()` with default value |
| `Merge lost data` | Overwrite instead of merge | Use deep merge function |

#### Debug Config Issues
```rust
// Always backup before writing
let backup_path = config_path.with_extension("json.backup");
std::fs::copy(&config_path, &backup_path)?;

// Log the merge operation
log::info!("Merging Ollama config: model={}, port={}", model_name, port);
```

### Phase 3: UI Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Cannot read property of undefined` | Store not initialized | Check store default value |
| `Step not advancing` | Async not awaited | Add `await` to async calls |
| `Progress bar stuck` | Event not emitted | Check Tauri event channel |
| `Button not clickable` | Loading state stuck | Add timeout/error handling |
| `Styles not applied` | Tailwind not configured | Check `tailwind.config.js` |

#### Debug Svelte/Reactivity
```typescript
// Add console logging to stores
import { writable } from 'svelte/store';

function createLoggedStore<T>(name: string, initial: T) {
    const store = writable(initial);
    return {
        subscribe: store.subscribe,
        set: (v: T) => {
            console.log(`[${name}] set:`, v);
            store.set(v);
        },
        update: store.update
    };
}

export const setupStep = createLoggedStore('setupStep', 1);
```

### Phase 5: System Tray Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Tray icon not showing` | Wrong icon path | Use absolute path in config |
| `Menu click no response` | Event not connected | Check `on_system_tray_event` |
| `App quits on window close` | Wrong exit behavior | Override close event |
| `Tray menu not updating` | Static menu | Recreate menu dynamically |

---

## Testing Failures

### Unit Test Failures

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run specific test
cargo test test_config_merge -- --nocapture

# Check with more verbose output
cargo test -- --nocapture
```

### Integration Test Failures

| Failure | Check |
|---------|-------|
| HTTP timeout | Mock server running? |
| File not found | Temp dir created? |
| Process error | Binary path correct? |
| Permission denied | Test running as correct user? |

### E2E Test Failures

| Failure | Check |
|---------|-------|
| App won't start | Check console for JS errors |
| Navigation fails | Check route configuration |
| Backend command fails | Check Rust console output |
| UI elements missing | Check component mounts |

---

## Platform-Specific Issues

### Windows

| Issue | Solution |
|-------|----------|
| Windows Defender blocks | Add exclusion for dev folder |
| Long path errors | Enable long path support in Windows |
| MSVC not found | Install VS Build Tools with C++ workload |
| PowerShell execution policy | Run `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned` |

### macOS

| Issue | Solution |
|-------|----------|
| Xcode tools missing | Run `xcode-select --install` |
| Gatekeeper blocks | Run `xattr -cr /path/to/app.app` |
| Notarization fails | Check signing certificates |
| Rosetta issues on ARM | Install Rosetta: `softwareupdate --install-rosetta` |

---

## Log Locations

When debugging, check these locations:

### Application Logs
- **Windows:** `%APPDATA%/HandyLauncher/logs/launcher.log`
- **macOS:** `~/Library/Logs/HandyLauncher/launcher.log`

### Ollama Logs
- **Windows:** `%USERPROFILE%/.ollama/logs/server.log`
- **macOS:** `~/.ollama/logs/server.log`

### Tauri Dev Console
- Open DevTools with `Ctrl+Shift+I` (Windows) or `Cmd+Option+I` (macOS)
- Check Console tab for JS errors
- Check Network tab for API calls

### Rust Console Output
- Run with `RUST_LOG=debug` for verbose output
- Check terminal where `bun run tauri dev` runs

---

## Recovery Procedures

### Reset Everything (Clean Slate)
```bash
# Stop Ollama
pkill ollama  # macOS/Linux
taskkill /F /IM ollama.exe  # Windows

# Remove Handy Launcher config
rm -rf ~/Library/Application\ Support/HandyLauncher  # macOS
Remove-Item -Recurse "$env:APPDATA/HandyLauncher"  # Windows

# Remove Handy config backup
rm ~/Library/Application\ Support/com.pais.handy/settings_store.json.backup

# Restart fresh
```

### Restore Handy Config from Backup
```bash
# macOS
cp ~/Library/Application\ Support/com.pais.handy/settings_store.json.backup \
   ~/Library/Application\ Support/com.pais.handy/settings_store.json

# Windows
Copy-Item "$env:APPDATA/com.pais.handy/settings_store.json.backup" \
          "$env:APPDATA/com.pais.handy/settings_store.json"
```

### Reinstall Ollama via Launcher
1. Stop Ollama process
2. Delete Ollama binary from app data dir
3. Restart Handy Launcher
4. Go through setup wizard again

---

## Getting Help

If stuck for > 30 minutes:
1. Check `IMPLEMENTATION.md` for phase-specific guidance
2. Search existing issues in Tauri/Ollama repos
3. Review `findings.md` for similar problems
4. Document the issue in `progress.md` with:
   - Exact error message
   - Steps to reproduce
   - What you've tried
   - Environment details (OS, versions)

---

## Debug Mode Toggle

Add this to enable verbose debugging:

**Rust (main.rs)**
```rust
fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "debug");
    }
    tauri::Builder::default()
        // ...
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Frontend (app.html)**
```html
<script>
  if (import.meta.env.DEV) {
    window.enableDebugMode = () => {
      localStorage.setItem('debug', 'true');
      location.reload();
    };
  }
</script>
```

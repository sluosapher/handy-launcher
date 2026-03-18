# Handy Launcher - Implementation Guide

**Version:** 1.0  
**Date:** March 17, 2026  
**Status:** Ready for Implementation

---

## Session Recovery Protocol

When resuming work in a new session:
1. Read this file (`IMPLEMENTATION.md`)
2. Read `task_plan.md` to see current phase
3. Read `progress.md` for recent session history
4. Run `git status` to check uncommitted changes
5. Read `DEBUGGING.md` if encountering issues
6. Continue from the **Current Phase** listed in task_plan.md

---

## Project Overview

**Goal:** Build Handy Launcher - a Tauri-based desktop app that automates local LLM setup for the Handy voice-to-text application.

**Stack:**
- **Backend:** Rust (Tauri)
- **Frontend:** Svelte + TypeScript
- **Build Tool:** Bun
- **Target Platforms:** Windows 10/11, macOS (Intel + Apple Silicon)

---

## Pre-Implementation Checklist

Before starting Phase 0, verify:
- [ ] Rust installed (`rustc --version` shows 1.75+)
- [ ] Node.js 20+ installed
- [ ] Bun installed (`bun --version` shows 1.0+)
- [ ] Windows: Visual Studio Build Tools or full VS 2022
- [ ] macOS: Xcode Command Line Tools

---

## Phase 0: Project Scaffold & Environment Setup

### Goal
Initialize the Tauri project with all dependencies and folder structure.

### Files to Create/Modify
```
src-tauri/
  src/
    main.rs              # Entry point
    lib.rs               # Module exports
    commands/            # Tauri command handlers
      mod.rs
      ollama.rs          # Ollama management commands
      config.rs          # Handy configuration commands
      system.rs          # System info commands
    services/            # Business logic
      mod.rs
      ollama_manager.rs  # Ollama process lifecycle
      config_manager.rs  # Handy settings.json I/O
      model_profiles.rs  # Model definitions
    models/              # Data structures
      mod.rs
      app_state.rs       # Application state
      ollama.rs          # Ollama API types
      config.rs          # Configuration schemas
    utils/               # Helpers
      mod.rs
      paths.rs           # Cross-platform path utilities
      http.rs            # HTTP client wrapper
  Cargo.toml
  tauri.conf.json
  capabilities/default.json
src/                     # Frontend
  routes/
    +page.svelte         # Setup wizard
    status/
      +page.svelte       # Status dashboard
  lib/
    stores.ts            # Svelte stores
    api.ts               # Backend API wrapper
  app.html
  app.d.ts
package.json
tsconfig.json
svelte.config.js
tailwind.config.js
```

### Implementation Steps

1. **Initialize Tauri Project**
   ```bash
   bun create tauri-app@latest handy-launcher --template svelte-ts
   cd handy-launcher
   ```

2. **Install Frontend Dependencies**
   ```bash
   bun install
   bun add -d @tauri-apps/cli
   bun add lucide-svelte  # Icons
   bun add @tauri-apps/api
   ```

3. **Configure Tailwind CSS**
   ```bash
   bun add -d tailwindcss postcss autoprefixer
   bunx tailwindcss init -p
   ```
   Update `tailwind.config.js` with dark mode and custom colors per `ui-specifications.md`.

4. **Setup Rust Project Structure**
   Create all module files in `src-tauri/src/` as listed above.

5. **Configure Tauri Capabilities**
   Update `src-tauri/capabilities/default.json` with:
   - `shell:allow-execute` (for running Ollama binary)
   - `fs:allow-read/write` (for Handy settings.json)
   - `os:allow-platform` (for platform detection)

### Testing Criteria
- [ ] `bun run tauri dev` launches app without errors
- [ ] App window shows Svelte starter page
- [ ] Dev console shows no JavaScript errors
- [ ] Rust compilation completes without warnings

### Success Exit
Proceed to Phase 1 when:
- Tauri dev mode runs successfully
- All folder structure created
- No compiler errors

---

## Phase 1: Backend Core - Ollama Management

### Goal
Implement Rust modules for Ollama lifecycle management: detection, download, installation, process control.

### Implementation Steps

1. **Define Data Models** (`src-tauri/src/models/`)
   - `OllamaStatus` enum: NotInstalled, Installing, Ready, Error
   - `InstallProgress` struct: percent, status message
   - `ModelProfile` struct: name, size, ram_required, description

2. **Implement Ollama Manager** (`src-tauri/src/services/ollama_manager.rs`)
   Functions to implement:
   ```rust
   pub async fn check_ollama_installed() -> bool
   pub async fn download_ollama() -> Result<InstallProgress, DownloadError>
   pub async fn install_ollama(installer_path: PathBuf) -> Result<(), InstallError>
   pub async fn start_ollama(port: u16) -> Result<Child, ProcessError>
   pub async fn stop_ollama(child: &mut Child) -> Result<(), ProcessError>
   pub async fn verify_ollama_running(port: u16) -> bool
   ```

3. **Implement Platform-Specific Installers**
   - Windows: Download `.exe` from Ollama releases, run silent install
   - macOS: Download `.tgz`, extract to `~/Applications` or app data dir

4. **Implement HTTP Client** (`src-tauri/src/utils/http.rs`)
   - Wrapper around `reqwest` for Ollama API calls
   - Retry logic with exponential backoff
   - Timeout configuration

5. **Expose Tauri Commands** (`src-tauri/src/commands/ollama.rs`)
   ```rust
   #[tauri::command]
   async fn check_ollama_status() -> OllamaStatus
   
   #[tauri::command]
   async fn install_ollama() -> Result<InstallProgress, String>
   
   #[tauri::command]
   async fn start_ollama_server(port: u16) -> Result<(), String>
   ```

### Testing Criteria
- [ ] Unit test: `check_ollama_installed()` returns correct status
- [ ] Integration test: Download Ollama (skip if already installed)
- [ ] Integration test: Start/stop Ollama process
- [ ] Verify Ollama responds on `http://127.0.0.1:{port}/api/version`

### Debug Strategies
| Issue | Check | Fix |
|-------|-------|-----|
| Ollama not found after install | PATH env var | Use full path to binary |
| Port already in use | `netstat -ano \| findstr :63452` | Auto-increment port |
| Download fails | Network/proxy | Add retry logic, mirror fallback |
| Permission denied | Admin rights | Install to user data dir |

### Success Exit
- Ollama installs and starts programmatically
- HTTP API responds with version
- All unit tests pass

---

## Phase 2: Configuration Manager

### Goal
Implement Handy's settings.json read/write with backup/restore capability.

### Implementation Steps

1. **Define Configuration Schemas** (`src-tauri/src/models/config.rs`)
   ```rust
   pub struct HandyConfig {
       pub post_process_provider_id: String,
       pub post_process_providers: HashMap<String, ProviderConfig>,
       pub post_process_models: HashMap<String, String>,
       pub post_process_selected_prompt_id: String,
       pub post_process_prompts: HashMap<String, String>,
   }
   ```

2. **Implement Config Manager** (`src-tauri/src/services/config_manager.rs`)
   ```rust
   pub fn read_handy_config() -> Result<HandyConfig, ConfigError>
   pub fn write_handy_config(config: &HandyConfig) -> Result<(), ConfigError>
   pub fn backup_config() -> Result<PathBuf, ConfigError>
   pub fn restore_config(backup_path: PathBuf) -> Result<(), ConfigError>
   pub fn merge_ollama_config(
       existing: &mut HandyConfig,
       model_name: &str,
       port: u16
   ) -> HandyConfig
   ```

3. **Handle Platform Paths** (`src-tauri/src/utils/paths.rs`)
   ```rust
   pub fn get_handy_settings_path() -> PathBuf {
       // Windows: %APPDATA%/com.pais.handy/settings_store.json
       // macOS: ~/Library/Application Support/com.pais.handy/settings_store.json
   }
   ```

4. **Implement JSON Merge Logic**
   - Backup existing settings.json before any modification
   - Deep merge nested objects (providers, prompts)
   - Preserve all existing user settings
   - Write atomically (temp file + rename)

5. **Expose Tauri Commands** (`src-tauri/src/commands/config.rs`)
   ```rust
   #[tauri::command]
   fn get_handy_config() -> Result<HandyConfig, String>
   
   #[tauri::command]
   fn configure_handy_with_ollama(model_name: String, port: u16) -> Result<(), String>
   ```

### Testing Criteria
- [ ] Unit test: Read/write Handy config preserves all fields
- [ ] Unit test: Backup/restore cycle works
- [ ] Integration test: Merge adds Ollama config without losing existing data
- [ ] Test: Invalid JSON returns proper error

### Debug Strategies
| Issue | Check | Fix |
|-------|-------|-----|
| File not found | Handy not installed | Show "Install Handy first" message |
| Permission denied | File locked | Close Handy app before modifying |
| JSON parse error | Corrupted file | Restore from backup |
| Config not applied | Handy caching | Restart Handy app |

### Success Exit
- Can read/write Handy settings.json
- Backup/restore works reliably
- All integration tests pass

---

## Phase 3: Setup Wizard UI

### Goal
Build the 3-step setup wizard with auto-advance logic.

### Implementation Steps

1. **Create Svelte Stores** (`src/lib/stores.ts`)
   ```typescript
   export const setupStep = writable<1 | 2 | 3>(1);
   export const ollamaStatus = writable<OllamaStatus>('checking');
   export const installProgress = writable<InstallProgress | null>(null);
   export const selectedModel = writable<ModelProfile | null>(null);
   export const systemInfo = writable<SystemInfo | null>(null);
   ```

2. **Step 1: Welcome + System Check** (`src/routes/+page.svelte`)
   - Welcome message
   - System check button
   - Display RAM, CPU, disk space
   - Auto-check on mount

3. **Step 2: Ollama Installation** (conditional)
   - If Ollama not installed: Show install button
   - If installing: Progress bar with status
   - If installed: Auto-advance to Step 3
   - "Troubleshooting" button for manual setup

4. **Step 3: Model Selection**
   - Three profile cards: Fast, Recommended, Accurate
   - Gray out unsuitable models based on RAM
   - Show download size, RAM required, expected speed
   - Download button with progress
   - Auto-advance on completion

5. **Final Step: Configuration** (auto-execute)
   - Merge Handy config
   - Show success screen with "Open Handy" button
   - "View Logs" button for troubleshooting

6. **Add Auto-Advance Logic**
   - After Step 2 completes: `setTimeout(() => step = 3, 500)`
   - After model download: Auto-advance to success

### Testing Criteria
- [ ] E2E test: Complete wizard from fresh state
- [ ] E2E test: Wizard detects existing Ollama, skips to Step 3
- [ ] UI test: Grayed models show warning tooltip
- [ ] UI test: Progress bars animate smoothly
- [ ] Test: Cancel download returns to model selection

### Debug Strategies
| Issue | Check | Fix |
|-------|-------|-----|
| UI not updating | Svelte reactivity | Use `$store` syntax |
| Step not advancing | Async completion | Add `await` before transition |
| Progress stuck | Backend stream | Check event emitter |
| Model card not gray | RAM calculation | Verify `systemInfo` populated |

### Success Exit
- All 3 steps render correctly
- Auto-advance works
- Can complete full setup in < 5 minutes

---

## Phase 4: Status Dashboard UI

### Goal
Build the post-setup dashboard showing Ollama status, model info, controls.

### Implementation Steps

1. **Create Status Route** (`src/routes/status/+page.svelte`)
   - Header: App title + settings icon
   - Main card: Current model + status badge
   - Stats row: Uptime, memory usage, last activity
   - Action buttons: Start/Stop Ollama, Test Connection, Switch Model

2. **Implement Real-Time Status**
   - Poll Ollama `/api/tags` every 5 seconds when running
   - Update `ollamaStatus` store
   - Show green/yellow/red status indicator

3. **Add Control Buttons**
   - Start Ollama: Calls `start_ollama_server()`
   - Stop Ollama: Calls `stop_ollama()`
   - Test Connection: Calls `test_ollama_connection()`
   - Switch Model: Navigate back to setup wizard Step 3

4. **Add Logs Viewer**
   - "View Logs" button opens log file location
   - Optional: In-app log viewer with tail -f behavior

### Testing Criteria
- [ ] Status updates when Ollama starts/stops
- [ ] Test connection shows response time
- [ ] Switch model navigates to wizard
- [ ] All buttons have loading states

### Debug Strategies
| Issue | Check | Fix |
|-------|-------|-----|
| Status not updating | Polling interval | Check `setInterval` |
| Wrong status shown | State sync | Verify store updates |
| Button unresponsive | Command error | Add try/catch + toast |

---

## Phase 5: System Tray & Background Operation

### Goal
Implement system tray icon with menu and background Ollama management.

### Implementation Steps

1. **Configure Tauri System Tray** (`src-tauri/tauri.conf.json`)
   ```json
   {
     "systemTray": {
       "iconPath": "icons/icon.png",
       "iconAsTemplate": true
     }
   }
   ```

2. **Implement Tray Menu** (`src-tauri/src/main.rs`)
   ```rust
   let tray_menu = SystemTrayMenu::new()
       .add_item(CustomMenuItem::new("show", "Show Handy Launcher"))
       .add_item(CustomMenuItem::new("status", "Ollama Status: Stopped"))
       .add_native_item(SystemTrayMenuItem::Separator)
       .add_item(CustomMenuItem::new("quit", "Quit"));
   ```

3. **Handle Tray Events**
   - Left click: Show/hide main window
   - "Show" menu: Focus main window
   - "Status" menu: Toggle Ollama start/stop
   - "Quit": Stop Ollama, exit app

4. **Background Operation**
   - Keep Ollama running when window closed
   - Monitor Ollama health in background
   - Restart Ollama if it crashes
   - Update tray icon based on status

5. **Window Behavior**
   - Close button hides to tray (not quit)
   - Add "Exit" menu item in app menu
   - Remember window position

### Testing Criteria
- [ ] App shows in system tray on startup
- [ ] Close window keeps app running
- [ ] Tray menu items work
- [ ] Quit from tray stops Ollama and exits
- [ ] Ollama continues running when window hidden

### Debug Strategies
| Issue | Check | Fix |
|-------|-------|-----|
| Tray icon not showing | Icon path | Use absolute path |
| Menu not working | Event handler | Check `on_system_tray_event` |
| Ollama stops on hide | Process scope | Ensure Child is in main scope |
| Can't quit app | Event loop | Call `app.exit()` |

---

## Phase 6: Testing & Debugging Framework

### Goal
Comprehensive test suite and debugging tools.

### Implementation Steps

1. **Unit Tests** (`src-tauri/src/`)
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_config_merge() {
           // Test merging Ollama config into Handy settings
       }
       
       #[test]
       fn test_model_profile_selection() {
           // Test RAM-based model filtering
       }
   }
   ```

2. **Integration Tests**
   - Test Ollama download (mock or real)
   - Test Handy config read/write (use temp files)
   - Test HTTP client with mock server

3. **E2E Tests** (manual checklist in `testing.md`)
   - Fresh install flow
   - Existing Ollama detection
   - Model download and verification
   - Port conflict handling
   - System tray behavior

4. **Add Debug Logging**
   - Use `log` crate in Rust with `fern` for file output
   - Log levels: ERROR, WARN, INFO, DEBUG
   - Log file rotation at 10MB

5. **Create Debug Panel** (optional UI feature)
   - Hidden behind "Troubleshooting Mode"
   - Show raw Ollama logs
   - API request/response viewer
   - State inspector

### Testing Criteria
- [ ] `cargo test` passes all unit tests
- [ ] Manual E2E checklist completed
- [ ] Logs written to correct location
- [ ] Debug panel shows internal state

---

## Phase 7: Build & Package

### Goal
Production builds for Windows and macOS with code signing.

### Implementation Steps

1. **Configure Tauri Bundle** (`src-tauri/tauri.conf.json`)
   ```json
   {
     "bundle": {
       "active": true,
       "targets": ["msi", "dmg"],
       "identifier": "com.pais.handy-launcher",
       "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns"]
     }
   }
   ```

2. **Windows Build**
   ```bash
   bun run tauri build --target x86_64-pc-windows-msvc
   ```
   - Produces `.msi` installer
   - Optional: Code sign with Authenticode

3. **macOS Build**
   ```bash
   bun run tauri build --target x86_64-apple-darwin
   bun run tauri build --target aarch64-apple-darwin
   ```
   - Produces `.dmg` for each architecture
   - Universal binary with `lipo`
   - Code sign and notarize

4. **CI/CD Setup** (GitHub Actions)
   - Build matrix: Windows, macOS Intel, macOS ARM
   - Artifact upload
   - Release creation on tag

5. **Auto-Updater** (optional)
   - Configure Tauri updater
   - Set up update server or use GitHub releases

### Testing Criteria
- [ ] MSI installer works on fresh Windows VM
- [ ] DMG mounts and app runs on macOS
- [ ] App passes basic smoke test after install
- [ ] No antivirus false positives

---

## Common Errors & Solutions

See `DEBUGGING.md` for detailed troubleshooting guide.

---

## Implementation Order Summary

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| 0. Scaffold | 1-2 hrs | None |
| 1. Ollama Manager | 4-6 hrs | Phase 0 |
| 2. Config Manager | 3-4 hrs | Phase 0 |
| 3. Setup Wizard | 4-6 hrs | Phase 1, 2 |
| 4. Status Dashboard | 3-4 hrs | Phase 1, 2 |
| 5. System Tray | 2-3 hrs | Phase 1 |
| 6. Testing | 3-4 hrs | All above |
| 7. Build | 2-3 hrs | All above |

**Total Estimated Time:** 20-30 hours

---

## Cross-Reference Documentation

- **Requirements:** `docs/requirements-analysis.md`
- **Architecture:** `docs/architecture/system-architecture.md`
- **API Specs:** `docs/api/ollama-integration.md`
- **UI Specs:** `docs/user-guides/ui-specifications.md`
- **Testing:** `docs/development/testing.md`
- **Setup:** `docs/development/setup.md`
- **Build:** `docs/development/build.md`

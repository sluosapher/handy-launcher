# System Architecture Document
## Handy Launcher

**Version:** 1.1  
**Date:** March 16, 2026  
**Status:** Draft (Improvements Applied)

---

## 1. Overview

Handy Launcher is a lightweight desktop companion application built with Tauri that automates the setup and management of Ollama for local LLM-powered transcription post-processing in the Handy voice application.

### 1.1 Philosophy
- **Lightweight**: Minimal resource footprint, fast startup
- **Simple**: Zero configuration required from users
- **Safe**: Non-destructive configuration changes with backups
- **Self-contained**: No administrative privileges required

---

## 2. Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| **Frontend Framework** | Svelte | Minimal runtime, compiled output, ideal for utility apps |
| **Desktop Framework** | Tauri (Rust) | Native performance, small bundle size (<5MB), cross-platform |
| **State Management** | Tauri native store | Simple key-value storage, async API, auto-persisted |
| **HTTP Client** | reqwest (Rust) | Async HTTP for Ollama API communication |
| **Process Management** | std::process (Rust) | Native process spawning and monitoring |
| **Distribution** | Direct download | Website-hosted `.msi` (Windows) and `.dmg` (macOS) |

---

## 3. System Architecture

### 3.1 High-Level Diagram

```
+-------------------------------------------------------------+
|                    Svelte Frontend                           |
|  +--------------+  +--------------+  +-----------------+   |
|  | Setup Wizard |  | Status Dash  |  |  Log Viewer     |   |
|  |              |  |              |  |                 |   |
|  +--------------+  +--------------+  +-----------------+   |
+-------------------------------------------------------------+
                         | Commands / Events
+------------------------+------------------------------------+
|                  Tauri Backend (Rust)                        |
|  +----------------+  +------------------+  +-------------+   |
|  | Ollama Manager |  | Handy Config Mgr |  | State Store |   |
|  |                |  |                  |  |             |   |
|  | - Download     |  | - JSON merge     |  | - Settings  |   |
|  | - Install      |  | - Backup/restore |  | - Model cfg |   |
|  | - Process ctrl |  | - Validation     |  | - Logs      |   |
|  +----------------+  +------------------+  +-------------+   |
+-------------------------------------------------------------+
                         |
         +---------------+---------------+
         |               |               |
+--------v-----+  +------v------+  +----v------+
|   Ollama     |  | Handy Config|  |  Logs     |
|   Binary     |  | settings_*.json|  |  Files    |
|              |  |              |  |           |
| User-local   |  | JSON merge   |  | Rotation  |
| install      |  | strategy     |  | Export    |
+--------------+  +-------------+  +-----------+
```

### 3.2 Component Details

#### 3.2.1 Ollama Manager (Rust)
**Responsibilities:**
- Detect existing Ollama installation
- Download and install Ollama to user data directory (`%APPDATA%/HandyLauncher/ollama`)
- Start/stop Ollama process on demand
- Download model profiles via Ollama HTTP API
- Monitor process health

**Key Behaviors:**
- Installation is **user-local**, no admin required
- Process lifecycle is **manual** - Ollama keeps running until user stops it or system reboots
- Port discovery: attempts 63452, auto-increments if unavailable

#### 3.2.2 Handy Configuration Manager (Rust)
**Responsibilities:**
- Locate Handy configuration directory
- Read and parse `settings_*.json` files
- Merge Ollama configuration non-destructively
- Create timestamped backups before modification
- Validate JSON structure before and after writes

**Key Behaviors:**
- Backup naming: `settings_backup_YYYYMMDD_HHMMSS.json`
- Merge strategy: Deep merge preserving user settings
- Atomic writes: Write to temp file, then rename
- Restore on failure: Automatic rollback on any error

**Implementation: Handy Detection**

```rust
// src/managers/config_manager.rs

use std::path::{Path, PathBuf};
use dirs;
use sysinfo::{System, SystemExt, ProcessExt};

/// Result of Handy installation detection
pub struct HandyDetection {
    pub is_installed: bool,
    pub config_path: Option<PathBuf>,
    pub is_running: bool,
    pub install_method: InstallMethod,
}

#[derive(Debug, Clone)]
pub enum InstallMethod {
    Store,      // Microsoft Store / Mac App Store
    Direct,     // Downloaded from pa.is/handy
    Unknown,
}

impl ConfigManager {
    /// Primary detection: Check config directory existence
    pub fn locate_handy_config() -> Result<PathBuf, LauncherError> {
        let config_dir = if cfg!(target_os = "windows") {
            // C:\Users\{user}\AppData\Roaming\com.pais.handy
            dirs::data_dir()
                .ok_or(LauncherError::ConfigNotFound)?
                .join("com.pais.handy")
        } else if cfg!(target_os = "macos") {
            // ~/Library/Application Support/com.pais.handy
            dirs::home_dir()
                .ok_or(LauncherError::ConfigNotFound)?
                .join("Library/Application Support/com.pais.handy")
        } else {
            return Err(LauncherError::UnsupportedPlatform);
        };

        let config_path = config_dir.join("settings_store.json");
        
        // Config dir exists = Handy installed (even if settings file does not exist yet)
        if config_dir.exists() {
            Ok(config_path)
        } else {
            Err(LauncherError::HandyNotInstalled)
        }
    }

    /// Application-level detection (secondary check)
    #[cfg(target_os = "windows")]
    pub fn is_handy_app_installed() -> bool {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        // Check registry uninstall key
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let registry_check = hkcu
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Handy")
            .map(|_| true)
            .unwrap_or(false);

        // Check common install locations
        let program_files = PathBuf::from("C:\\Program Files\\Handy");
        let local_appdata = dirs::data_local_dir().map(|d| d.join("Handy"));

        registry_check 
            || program_files.exists() 
            || local_appdata.map(|p| p.exists()).unwrap_or(false)
    }

    #[cfg(target_os = "macos")]
    pub fn is_handy_app_installed() -> bool {
        let system_app = PathBuf::from("/Applications/Handy.app");
        let user_app = dirs::home_dir()
            .map(|h| h.join("Applications/Handy.app"))
            .unwrap_or_default();

        system_app.exists() || user_app.exists()
    }

    /// Check if Handy process is currently running
    pub fn is_handy_running() -> bool {
        let mut system = System::new();
        system.refresh_all();

        // Check for Handy process (case-insensitive, exclude launcher)
        system.processes()
            .values()
            .any(|p| {
                let name = p.name().to_lowercase();
                name.contains("handy") && !name.contains("launcher")
            })
    }

    /// Complete detection for UI state
    pub fn detect_handy() -> HandyDetection {
        let config_result = Self::locate_handy_config();
        let is_installed = config_result.is_ok() || Self::is_handy_app_installed();
        let is_running = Self::is_handy_running();

        HandyDetection {
            is_installed,
            config_path: config_result.ok(),
            is_running,
            install_method: InstallMethod::Unknown,
        }
    }
}
```

**Detection Edge Cases:**

| Scenario | Detection Result | Action |
|----------|-----------------|--------|
| Handy never installed | `is_installed: false` | Show download prompt |
| Handy installed, first run (no settings.json) | `is_installed: true, config_path: Some` | Create settings file on config |
| Handy running | `is_running: true` | Modal: "Please close Handy first" |
| Handy uninstalled, config remains | `is_installed: true` (false positive) | Detect stale config by app check |
| Multiple Handy versions | ambiguous | Warn user, pick most recent |

#### 3.2.3 State Store (Tauri Plugin)
**Responsibilities:**
- Persist application settings across sessions
- Store model profile preferences
- Cache Ollama status and health
- Log rotation configuration

---

## 4. Data Flow

### 4.1 Setup Wizard Flow

```
User clicks "Start Setup"
         |
         v
+-----------------+
|  Frontend:      |
|  SetupWizard    |
|  .svelte        |
+-----------------+
         | invoke("check_system_requirements")
         v
+-----------------+     +-----------------+
|  Backend:       |---->|  System check   |
|  system.rs      |     |  (RAM, disk, OS)|
+-----------------+     +-----------------+
         | SystemInfo { ram_gb, disk_gb, os }
         v
+-----------------+
|  Frontend:      |
|  Display        |
|  system status  |
+-----------------+
         | User clicks "Continue"
         v
+-----------------+     +-----------------+
|  Backend:       |---->|  Ollama install   |
|  ollama.rs      |     |  (if needed)      |
+-----------------+     +-----------------+
         | InstallProgress { status, percent }
         v
+-----------------+     +-----------------+
|  Backend:       |---->|  Model download   |
|  ollama.rs      |     |  (via Ollama API) |
+-----------------+     +-----------------+
         | DownloadProgress { status, percent }
         v
+-----------------+     +-----------------+
|  Backend:       |---->|  Handy config     |
|  config.rs      |     |  merge + backup   |
+-----------------+     +-----------------+
         | ConfigResult { success }
         v
+-----------------+
|  Frontend:      |
|  Complete       |
|  (auto-close)   |
+-----------------+
```

### 4.2 Configuration Merge Flow

```
+-------------------+
|  Read existing    |
|  settings.json    |
+-------------------+
          |
          v
+-------------------+
|  Validate JSON    |
|  structure        |
+-------------------+
          |
          v
+-------------------+
|  Create backup     |
|  (timestamped)   |
+-------------------+
          |
          v
+-------------------+
|  Deep merge with  |
|  Ollama config   |
+-------------------+
          |
          v
+-------------------+
|  Validate result  |
+-------------------+
          |
          v
+-------------------+
|  Atomic write     |
|  (temp + rename)  |
+-------------------+
          |
          v
+-------------------+
|  Verify readback  |
+-------------------+
```

---

## 5. State Management

### 5.1 Rust AppState

```rust
// src/models/state.rs

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub setup_status: SetupStatus,
    pub selected_model: Option<String>,
    pub ollama_port: Option<u16>,
    pub ollama_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupStatus {
    NotStarted,
    SystemCheckInProgress,
    SystemCheckComplete { passed: bool, warnings: Vec<String> },
    OllamaInstallInProgress,
    OllamaInstallComplete,
    ModelDownloadInProgress { model: String, progress: f32 },
    ModelDownloadComplete,
    ConfigMergeInProgress,
    Complete,
    Error { message: String, recoverable: bool },
}

// Thread-safe shared state
pub type SharedState = Arc<RwLock<AppState>>;
```

### 5.2 Tauri Store (Persistent)

```rust
// src/stores/launcher_store.rs

use tauri_plugin_store::StoreBuilder;

pub struct LauncherStore;

impl LauncherStore {
    pub fn init<R: Runtime>(app: &AppHandle<R>) -> Store<R> {
        StoreBuilder::new("launcher_settings.json")
            .default("first_run", true)
            .default("selected_profile", "recommended")
            .default("ollama_auto_start", true)
            .default("minimize_to_tray", true)
            .build(app)
    }
}
```

### 5.3 Svelte Stores (Frontend)

```typescript
// frontend/src/stores/setup.ts
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

// Core setup state
export const setupStatus = writable<SetupStatus>('not-started');
export const systemInfo = writable<SystemInfo | null>(null);
export const selectedProfile = writable<string>('recommended');
export const downloadProgress = writable<DownloadProgress | null>(null);

// Derived state
export const canProceed = derived(
    [setupStatus, systemInfo],
    ([$status, $info]) => {
        if ($status === 'system-check-complete') {
            return $info?.ram_gb >= 2; // Minimum 2GB RAM
        }
        return ['not-started', 'ollama-install-complete', 'model-download-complete'].includes($status);
    }
);

// Async actions
export async function startSystemCheck() {
    setupStatus.set('system-check-in-progress');
    try {
        const info = await invoke<SystemInfo>('check_system_requirements');
        systemInfo.set(info);
        setupStatus.set('system-check-complete');
    } catch (e) {
        setupStatus.set('error');
    }
}
```

---

## 6. Tauri Command Interface

### 6.1 Command Signatures

```rust
// src/commands/mod.rs

#[tauri::command]
pub async fn check_system_requirements(
    state: tauri::State<'_, SharedState>
) -> Result<SystemInfo, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn install_ollama(
    silent: bool,
    state: tauri::State<'_, SharedState>
) -> Result<InstallProgress, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn download_model(
    model_name: String,
    state: tauri::State<'_, SharedState>
) -> Result<DownloadProgress, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn configure_handy(
    model_name: String,
    port: u16,
    state: tauri::State<'_, SharedState>
) -> Result<(), LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn start_ollama(
    state: tauri::State<'_, SharedState>
) -> Result<OllamaStatus, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn stop_ollama(
    state: tauri::State<'_, SharedState>
) -> Result<(), LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn check_ollama_health(
    state: tauri::State<'_, SharedState>
) -> Result<OllamaStatus, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn list_downloaded_models(
    state: tauri::State<'_, SharedState>
) -> Result<Vec<ModelInfo>, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn get_system_info(
    state: tauri::State<'_, SharedState>
) -> Result<SystemInfo, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn get_logs(
    lines: Option<usize>
) -> Result<Vec<String>, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn export_logs() -> Result<PathBuf, LauncherError> {
    // Implementation
}

#[tauri::command]
pub async fn reset_configuration(
    state: tauri::State<'_, SharedState>
) -> Result<(), LauncherError> {
    // Implementation
}
```

### 6.2 Error Types

```rust
// src/models/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Ollama API error: {0}")]
    OllamaApiError(String),
    
    #[error("Model download failed: {0}")]
    ModelDownloadFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("System requirements not met: {0}")]
    SystemRequirementsNotMet(String),
    
    #[error("Ollama failed to start: {0}")]
    OllamaStartFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl serde::Serialize for LauncherError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
```

---

## 7. Configuration Schema

### 7.1 Handy Configuration Merge Strategy

**Target File:**
- Windows: `%APPDATA%/com.pais.handy/settings_store.json`
- macOS: `~/Library/Application Support/com.pais.handy/settings_store.json`

**Merge Strategy:**

```json
{
  "title": "Handy Configuration",
  "type": "object",
  "properties": {
    "llm": {
      "type": "object",
      "properties": {
        "provider": { "type": "string", "enum": ["openai", "anthropic", "custom"] },
        "model": { "type": "string" },
        "baseUrl": { "type": "string", "format": "uri" },
        "apiKey": { "type": "string" }
      }
    },
    "post_process_provider_id": { "type": "string" },
    "post_process_providers": {
      "type": "object",
      "additionalProperties": {
        "type": "object",
        "properties": {
          "base_url": { "type": "string" },
          "api_key": { "type": "string" }
        }
      }
    },
    "post_process_models": {
      "type": "object",
      "properties": {
        "custom": { "type": "string" }
      }
    },
    "post_process_selected_prompt_id": { "type": "string" },
    "post_process_prompts": {
      "type": "object",
      "additionalProperties": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "name": { "type": "string" },
          "content": { "type": "string" }
        }
      }
    }
  }
}
```

**Keys Modified by Launcher:**

| Key | Value | Description |
|-----|-------|-------------|
| `post_process_provider_id` | `"custom"` | Switch to custom provider |
| `post_process_providers.custom.base_url` | `"http://127.0.0.1:63452/v1"` | Ollama OpenAI-compatible endpoint |
| `post_process_models.custom` | `"llama3.2:1b"` | Selected model name |
| `post_process_selected_prompt_id` | `"handy_launcher_optimized"` | Our optimized prompt |
| `post_process_prompts.handy_launcher_optimized` | `{...}` | Grammar fix prompt |

**Example Merged Configuration:**

```json
{
  "user_preferences": {
    "theme": "dark",
    "language": "en"
  },
  "llm": {
    "provider": "openai",
    "model": "gpt-4"
  },
  "post_process_provider_id": "custom",
  "post_process_providers": {
    "custom": {
      "base_url": "http://127.0.0.1:63452/v1",
      "api_key": "ollama"
    }
  },
  "post_process_models": {
    "custom": "llama3.2:1b"
  },
  "post_process_selected_prompt_id": "handy_launcher_optimized",
  "post_process_prompts": {
    "handy_launcher_optimized": {
      "id": "handy_launcher_optimized",
      "name": "Handy Launcher Optimized",
      "content": "Fix grammar, punctuation, and formatting. Remove filler words like 'um' and 'uh'. Keep the original meaning and tone."
    }
  }
}
```

---

## 8. Project Structure

```
handy-launcher/
|-- src-tauri/              # Rust backend
|   |-- src/
|   |   |-- main.rs         # Entry point, Tauri setup
|   |   |-- lib.rs          # Module exports
|   |   |-- commands/       # Tauri command handlers (exposed to frontend)
|   |   |   |-- ollama.rs   # Ollama install/start/stop commands
|   |   |   |-- config.rs   # Handy configuration read/write
|   |   |   |-- system.rs   # OS-level utilities (paths, etc.)
|   |   |-- managers/       # Business logic
|   |   |   |-- ollama_manager.rs   # Ollama lifecycle
|   |   |   |-- config_manager.rs   # Handy settings management
|   |   |   |-- model_profiles.rs   # Model metadata (Light/Fast/Balanced)
|   |   |-- models/         # Data structures
|   |       |-- state.rs     # AppState struct
|   |       |-- config.rs    # Configuration types
|   |       |-- profiles.rs  # Model profile definitions
|-- frontend/               # Svelte frontend
|   |-- src/
|   |   |-- App.svelte      # Root component
|   |   |-- main.ts         # Entry point
|   |   |-- routes/         # Page components
|   |   |   |-- SetupWizard.svelte
|   |   |   |-- StatusDashboard.svelte
|   |   |-- components/     # Reusable UI
|   |   |   |-- ModelCard.svelte
|   |   |   |-- ProgressBar.svelte
|   |   |   |-- LogViewer.svelte
|   |   |-- stores/         # Svelte stores
|   |       |-- setup.ts
|   |       |-- ollama.ts
|   |-- package.json
|   |-- svelte.config.js
|-- docs/
|   |-- requirements-analysis.md
|   |-- architecture/
|       |-- system-architecture.md  # This file
|-- Cargo.toml              # Rust dependencies
|-- tauri.conf.json         # Tauri configuration
|-- package.json            # Root package.json (scripts)
|-- README.md
```

---

## 9. Security Considerations

### 9.1 Ollama Access Control
- Ollama binds to `127.0.0.1` (localhost only) on discovered port
- Port range 63452-63462 is unprivileged and unlikely to conflict
- No authentication required (Ollama default) - acceptable for local-only binding

### 9.2 Configuration Safety
- Always backup before modifying Handy settings
- Validate JSON structure before writing
- Atomic write operations (write to temp, then rename)
- Restore backup on any failure

### 9.3 Download Security
- Verify Ollama binary checksums against known values
- HTTPS for all downloads (Ollama official releases)
- No execution of user-provided scripts

### 9.4 Process Isolation
- Ollama runs as user process (not service/daemon)
- Launcher cannot elevate privileges
- User data directory permissions inherited from parent

---

## 10. Error Handling Strategy

| Error Scenario | Behavior |
|----------------|----------|
| Ollama download fails | Show retry + alternative download method (manual instructions) |
| Model download fails | Retry 3x, then suggest alternative profile |
| Port conflict (all 10 ports) | Show error, instruct user to free ports |
| Handy config write fails | Restore backup, show error with log path |
| Ollama process crash | Detect on next health check, offer restart |
| Disk space insufficient | Pre-check before download, show required space |

---

## 11. Future Considerations

### 11.1 Post-MVP Features
- Auto-updater (tauri-plugin-updater)
- Model management (delete unused models to free space)
- Custom model profiles (user-defined)
- Integration with system tray (background operation)
- GPU memory monitoring and warnings

### 11.2 Platform-Specific Enhancements
- Windows: Optional Start Menu shortcut
- macOS: Optional menubar integration
- Linux: Flatpak/AppImage distribution

---

## 12. Glossary

| Term | Definition |
|------|------------|
| **Ollama** | Local LLM server that manages model downloads and inference |
| **Handy** | Voice-to-text application that consumes this launcher's output |
| **Model Profile** | Preconfigured setting (Light/Fast/Balanced) mapping to specific Ollama models |
| **Post-processing** | LLM transformation of raw transcription output (grammar fix, formatting, etc.) |
| **JSON Merge** | Strategy of updating specific keys while preserving the rest of a JSON document |
| **Tauri Command** | Rust function exposed to frontend via invoke() API |
| **AppState** | Central Rust struct holding application runtime state |

---

## 13. References

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Svelte Documentation](https://svelte.dev/docs)
- [Tauri Plugin Store](https://github.com/tauri-apps/tauri-plugin-store)
- [JSON Schema](https://json-schema.org/)

---

*Document version: 1.1*  
*Last updated: March 16, 2026*  
*Authors: System Architecture Discussion + Improvements*

# Development Environment Setup Guide
## Handy Launcher

**Version:** 1.0  
**Date:** March 16, 2026  
**Status:** Draft

---

## 1. Prerequisites

### 1.1 Required Software

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Backend (Tauri + business logic) |
| Node.js | 20 LTS | Frontend build tooling |
| Bun | 1.0+ | Package manager (faster than npm) |
| Git | 2.40+ | Version control |

### 1.2 Platform-Specific Requirements

**Windows:**
- Windows 10 version 1809+ or Windows 11
- Microsoft Visual C++ Build Tools 2019 or 2022
- Windows SDK (10.0.22000.0 or later)

**macOS:**
- macOS 12+ (Monterey)
- Xcode Command Line Tools: `xcode-select --install`
- Rosetta 2 (for Apple Silicon, if testing Intel builds)

---

## 2. Installation Steps

### 2.1 Install Rust

```bash
# Via rustup (recommended)
curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or on Windows, download from https://rustup.rs/

# Verify installation
rustc --version  # Should show 1.75+
cargo --version
```

**Add WASM target (for Tauri):**
```bash
rustup target add wasm32-unknown-unknown
```

### 2.2 Install Node.js and Bun

**Option A: Via package manager**

```bash
# macOS (Homebrew)
brew install node
brew install oven-sh/bun/bun

# Windows (Winget)
winget install OpenJS.NodeJS
winget install Oven-sh.Bun

# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
curl -fsSL https://bun.sh/install | bash
```

**Option B: Via version manager (recommended for teams)**

```bash
# Install fnm (Fast Node Manager)
curl -fsSL https://fnm.vercel.app/install | bash
fnm install 20
fnm use 20

# Bun installs separately
curl -fsSL https://bun.sh/install | bash
```

**Verify:**
```bash
node --version  # v20.x.x
bun --version   # 1.0.x
```

### 2.3 Install Tauri CLI

```bash
cargo install tauri-cli

# Verify
cargo tauri --version
```

### 2.4 Platform-Specific Setup

**Windows - Visual Studio Build Tools:**

```powershell
# Install via Visual Studio Installer
# Required components:
# - MSVC v143 - VS 2022 C++ x64/x86 build tools
# - Windows 11 SDK (10.0.22000.0)
# - C++ CMake tools for Windows

# Or via winget (simpler)
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --add Microsoft.VisualStudio.Workload.VCTools"
```

**macOS - Xcode Command Line Tools:**

```bash
# Install
xcode-select --install

# Verify
clang --version
```

---

## 3. Project Setup

### 3.1 Clone Repository

```bash
git clone https://github.com/your-org/handy-launcher.git
cd handy-launcher
```

### 3.2 Install Frontend Dependencies

```bash
cd frontend
bun install

# Or if using npm
npm install
```

### 3.3 Install Rust Dependencies

```bash
# From project root
cargo fetch
```

### 3.4 Environment Configuration

Create `.env` file in project root:

```bash
# Development settings
RUST_LOG=debug
TAURI_DEV_WATCHER_IGNORE=frontend/node_modules

# Optional: Custom Ollama binary path for testing
# OLLAMA_TEST_BINARY=/path/to/ollama
```

---

## 4. Development Workflow

### 4.1 Start Development Server

Tauri provides a dev command that watches for changes and hot-reloads:

```bash
# From project root
cargo tauri dev

# Or with specific features
cargo tauri dev --features dev-tools
```

This will:
1. Start the Vite dev server for frontend (port 5173)
2. Compile Rust backend
3. Launch the Tauri application window
4. Watch for changes and reload automatically

**Expected output:**
```
[2026-03-16T10:00:00Z INFO] Starting development server...
[2026-03-16T10:00:02Z INFO] Frontend server started at http://localhost:5173
[2026-03-16T10:00:05Z INFO] Rust compilation complete
[2026-03-16T10:00:06Z INFO] Application window opened
```

### 4.2 Frontend-Only Development (Faster)

For UI work without Rust changes:

```bash
cd frontend
bun run dev

# Opens browser at http://localhost:5173
# Note: Tauri APIs won'''t work in browser
```

### 4.3 Backend-Only Development

For Rust development without UI:

```bash
# Run Rust tests in watch mode
cargo watch -x test

# Or specific test
cargo test ollama_manager -- --nocapture
```

### 4.4 Running Tests

```bash
# All Rust tests
cargo test

# With output
cargo test -- --nocapture

# Frontend tests
cd frontend
bun test

# Integration tests
cargo test --test integration
```

---

## 5. IDE Setup

### 5.1 VS Code (Recommended)

**Required Extensions:**
- rust-analyzer (Rust language support)
- Svelte for VS Code (Svelte syntax)
- Tailwind CSS IntelliSense (if using Tailwind)
- Even Better TOML (Cargo.toml editing)

**Recommended Settings (.vscode/settings.json):**

```json
{
  "rust-analyzer.cargo.features": ["tauri"],
  "rust-analyzer.procMacro.enable": true,
  "svelte.enable-ts-plugin": true,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode"
  }
}
```

**Launch Configuration (.vscode/launch.json):**

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tauri App",
      "cargo": {
        "args": ["build", "--manifest-path=src-tauri/Cargo.toml"],
        "filter": {
          "name": "handy-launcher",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### 5.2 JetBrains RustRover / IntelliJ

**Plugins:**
- Rust (official)
- TOML

**Setup:**
1. File u2192 New u2192 Project from Existing Sources
2. Select `Cargo.toml` in `src-tauri/`
3. Import as Cargo project

### 5.3 Vim/Neovim

**Recommended plugins:**
- rust-tools.nvim (Rust LSP)
- nvim-lspconfig (LSP configuration)
- nvim-treesitter (syntax highlighting)

**Minimal init.lua:**
```lua
require('lspconfig').rust_analyzer.setup({
  settings = {
    ['rust-analyzer'] = {
      cargo = { features = { 'tauri' } },
      procMacro = { enable = true },
    },
  },
})
```

---

## 6. Debugging

### 6.1 Frontend Debugging

**Browser DevTools:**
- Right-click in app window u2192 Inspect Element
- Or use Ctrl+Shift+I (Cmd+Option+I on macOS)

**Console Logging:**
```typescript
// In Svelte components
console.log('Debug:', variable);

// Tauri-specific logging
import { trace } from '@tauri-apps/api/log';
trace('Frontend trace message');
```

### 6.2 Backend Debugging

**VS Code:**
1. Set breakpoints in Rust code
2. Run "Debug Tauri App" from launch menu
3. Use Debug panel for stepping, variables, etc.

**Command Line:**
```bash
# Run with debug logging
RUST_LOG=debug cargo tauri dev

# Specific module logging
RUST_LOG=ollama_manager=trace cargo tauri dev
```

**Log Levels:**
- `error` - Critical failures
- `warn` - Recoverable issues
- `info` - General status
- `debug` - Detailed diagnostics
- `trace` - Verbose debugging

### 6.3 Tauri-Specific Debugging

**WebView Console:**
```rust
// In Rust, print to webview console
webview_window.eval("console.log(\'From Rust: \' + data)").unwrap();
```

**IPC Debugging:**
```rust
// Log all commands
#[tauri::command]
async fn my_command(payload: String) {
    log::debug!("Received command with payload: {}", payload);
    // ...
}
```

---

## 7. Common Issues

### 7.1 Build Failures

**Error: `linker link.exe not found` (Windows)**
```powershell
# Solution: Install Visual Studio Build Tools
# Or set linker explicitly in .cargo/config.toml
[target.x86_64-pc-windows-msvc]
linker = "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.35.32215\bin\Hostx64\x64\link.exe"
```

**Error: `library not found for -liconv` (macOS)**
```bash
# Solution: Reinstall Command Line Tools
sudo rm -rf /Library/Developer/CommandLineTools
xcode-select --install
```

**Error: `failed to run custom build command for webkit2gtk` (Linux)**
```bash
# Solution: Install webkit dependencies
sudo apt-get install libwebkit2gtk-4.0-dev
```

### 7.2 Frontend Issues

**Error: `Cannot find module @tauri-apps/api`**
```bash
# Solution: Reinstall frontend dependencies
cd frontend
rm -rf node_modules bun.lockb
bun install
```

**Hot reload not working:**
```bash
# Solution: Check file watcher limits (Linux)
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### 7.3 Runtime Issues

**App window opens but is blank:**
- Check frontend dev server is running (port 5173)
- Look for JavaScript errors in DevTools console
- Verify `vite.config.ts` has correct Tauri plugin configuration

**Tauri commands not responding:**
- Check Rust compilation succeeded
- Look for panics in terminal output
- Verify command is registered in `main.rs`:
  ```rust
  .invoke_handler(tauri::generate_handler![my_command])
  ```

---

## 8. Project Structure Overview

```
handy-launcher/
u251cu2500u2500 src-tauri/              # Rust backend
u2502   u251cu2500u2500 src/
u2502   u2502   u251cu2500u2500 main.rs         # Entry point
u2502   u2502   u251cu2500u2500 lib.rs          # Module exports
u2502   u2502   u251cu2500u2500 commands/       # Tauri command handlers
u2502   u2502   u251cu2500u2500 managers/       # Business logic
u2502   u2502   u2514u2500u2500 models/         # Data structures
u2502   u251cu2500u2500 Cargo.toml        # Rust dependencies
u2502   u2514u2500u2500 tauri.conf.json   # Tauri configuration
u251cu2500u2500 frontend/             # Svelte frontend
u2502   u251cu2500u2500 src/
u2502   u2502   u251cu2500u2500 App.svelte      # Root component
u2502   u2502   u251cu2500u2500 routes/         # Page components
u2502   u2502   u251cu2500u2500 components/     # Reusable UI
u2502   u2502   u2514u2500u2500 stores/         # Svelte stores
u2502   u251cu2500u2500 package.json
u2502   u2514u2500u2500 vite.config.ts
u251cu2500u2500 docs/                 # Documentation
u2514u2500u2500 README.md
```

**Key Files:**
- `src-tauri/tauri.conf.json` - App metadata, permissions, window config
- `src-tauri/Cargo.toml` - Rust dependencies
- `frontend/vite.config.ts` - Frontend build configuration
- `frontend/src/App.svelte` - Root Svelte component

---

## 9. Next Steps

After setup:

1. **Run the app:** `cargo tauri dev`
2. **Run tests:** `cargo test` and `cd frontend && bun test`
3. **Read architecture:** `docs/architecture/system-architecture.md`
4. **Check UI specs:** `docs/user-guides/ui-specifications.md`
5. **Build for production:** See `build.md`

---

## 10. Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Svelte Tutorial](https://svelte.dev/tutorial)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Bun Documentation](https://bun.sh/docs)

---

*Document created: March 16, 2026*

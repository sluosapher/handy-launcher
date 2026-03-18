# Task Plan: Handy Launcher Implementation

**Goal:** Implement Handy Launcher application following the comprehensive documentation.

**Approach:** Autonomous implementation with session recovery. Each phase is self-contained with clear entry/exit criteria.

---

## Session Recovery Checklist

When resuming work:
1. [ ] Read `IMPLEMENTATION.md` for current phase guidance
2. [ ] Read `task_plan.md` (this file) for phase status
3. [ ] Read `progress.md` for recent session history
4. [ ] Run `git status` to check uncommitted work
5. [ ] Verify environment: `rustc --version`, `bun --version`
6. [ ] Continue from **Current Phase** below

---

## Current Phase

**Phase:** 6 - Testing & Debugging Framework  
**Status:** In Progress  
**Started:** 2026-03-17  
**Last Session:** 2026-03-17 (Phase 6 debug panel slice implemented and verified)

---

## Implementation Phases

### Phase 0: Project Scaffold & Environment Setup
**Duration Estimate:** 1-2 hours  
**Dependencies:** None  
**Goal:** Initialize Tauri project with all dependencies and folder structure

#### Checklist
- [ ] Create Tauri project with Svelte+TS template
- [ ] Install frontend dependencies (lucide-svelte, @tauri-apps/api)
- [ ] Configure Tailwind CSS with dark mode
- [ ] Create Rust module structure (commands/, services/, models/, utils/)
- [ ] Configure Tauri capabilities (shell, fs, os permissions)
- [ ] Verify `bun run tauri dev` works

#### Exit Criteria
- [ ] App launches in dev mode without errors
- [ ] All folder structure created
- [ ] No compiler warnings
- [ ] Git initialized with initial commit

#### Files Created
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/src/main.rs`
- `src-tauri/src/lib.rs`
- Module files in `src-tauri/src/`
- `package.json`
- `tailwind.config.js`

---

### Phase 1: Backend Core - Ollama Management
**Duration Estimate:** 4-6 hours  
**Dependencies:** Phase 0  
**Goal:** Implement Ollama lifecycle: detection, download, installation, process control

#### Checklist
- [ ] Define data models (OllamaStatus, InstallProgress, ModelProfile)
- [ ] Implement `ollama_manager.rs` with core functions
- [ ] Implement platform-specific Ollama downloaders (Windows .exe, macOS .tgz)
- [ ] Implement HTTP client wrapper with retry logic
- [ ] Implement process start/stop with port management
- [ ] Expose Tauri commands for Ollama operations
- [ ] Write unit tests for core functions

#### Exit Criteria
- [ ] `check_ollama_installed()` works correctly
- [ ] Can download and install Ollama (or skip if exists)
- [ ] Can start/stop Ollama on specified port
- [ ] Ollama API responds to version check
- [ ] All unit tests pass

#### Files Created/Modified
- `src-tauri/src/models/ollama.rs`
- `src-tauri/src/models/app_state.rs`
- `src-tauri/src/services/ollama_manager.rs`
- `src-tauri/src/utils/http.rs`
- `src-tauri/src/utils/paths.rs`
- `src-tauri/src/commands/ollama.rs`

---

### Phase 2: Configuration Manager
**Duration Estimate:** 3-4 hours  
**Dependencies:** Phase 0  
**Goal:** Implement Handy settings.json read/write with backup/restore

#### Checklist
- [ ] Define HandyConfig schema with all required fields
- [ ] Implement config reader with JSON parsing
- [ ] Implement config writer with atomic write (temp + rename)
- [ ] Implement backup/restore functionality
- [ ] Implement JSON merge logic (deep merge, preserve user settings)
- [ ] Handle platform-specific paths (Windows %APPDATA%, macOS ~/Library)
- [ ] Expose Tauri commands for config operations
- [ ] Write unit tests for merge logic

#### Exit Criteria
- [ ] Can read Handy config from standard location
- [ ] Can write config without losing existing fields
- [ ] Backup created before every write
- [ ] Restore from backup works
- [ ] All unit tests pass

#### Files Created/Modified
- `src-tauri/src/models/config.rs`
- `src-tauri/src/services/config_manager.rs`
- `src-tauri/src/utils/paths.rs` (add Handy paths)
- `src-tauri/src/commands/config.rs`

---

### Phase 3: Setup Wizard UI
**Duration Estimate:** 4-6 hours  
**Dependencies:** Phase 1, Phase 2  
**Goal:** Build 3-step setup wizard with auto-advance logic

#### Checklist
- [x] Create Svelte stores (setupStep, ollamaStatus, installProgress, etc.)
- [x] Build Step 1: Welcome + System Check UI
- [x] Build Step 2: Ollama Installation UI (conditional display)
- [x] Build Step 3: Model Selection with 3 profiles
- [x] Implement auto-advance logic between steps
- [x] Implement progress tracking for downloads
- [x] Add "gray out unsuitable models" based on RAM
- [x] Add troubleshooting button for manual setup
- [x] Add success screen with "Open Handy" button

#### Exit Criteria
- [x] All 3 steps render correctly
- [x] System check displays RAM/CPU info
- [x] Auto-advance works after installation
- [x] Model selection shows correct info (size, RAM, speed)
- [x] Can complete full wizard flow

#### Files Created/Modified
- `src/lib/stores.ts`
- `src/lib/api.ts`
- `src/routes/+page.svelte` (main wizard)
- `src/lib/components/SystemCheck.svelte`
- `src/lib/components/ModelSelector.svelte`
- `src/lib/components/ProgressBar.svelte`

---

### Phase 4: Status Dashboard UI
**Duration Estimate:** 3-4 hours  
**Dependencies:** Phase 1, Phase 2  
**Goal:** Build post-setup dashboard with Ollama status and controls

#### Checklist
- [x] Create status route and layout
- [x] Implement status polling (every 5 seconds)
- [x] Build status card with current model info
- [x] Add Start/Stop Ollama buttons
- [x] Add Test Connection button
- [x] Add Switch Model button (link to wizard Step 3)
- [x] Add View Logs button
- [x] Implement real-time status indicator

#### Exit Criteria
- [x] Dashboard shows current Ollama status
- [x] Status updates when Ollama starts/stops
- [x] Test connection shows response time
- [x] Can navigate to model switcher
- [x] All buttons have loading states

#### Files Created/Modified
- `src/routes/status/+page.svelte`
- `src/lib/components/StatusCard.svelte`
- `src/lib/components/ActionButton.svelte`

---

### Phase 5: System Tray & Background Operation
**Duration Estimate:** 2-3 hours  
**Dependencies:** Phase 1  
**Goal:** Implement system tray with menu and background Ollama management

#### Checklist
- [x] Configure Tauri system tray in `tauri.conf.json`
- [x] Create tray menu (Show, Status, Separator, Quit)
- [x] Handle tray left-click (show/hide window)
- [x] Handle menu item clicks
- [x] Implement background Ollama monitoring
- [x] Auto-restart Ollama if it crashes
- [ ] Update tray icon based on status (optional)
- [x] Ensure close button hides to tray, not quit

#### Exit Criteria
- [x] Tray icon appears on startup
- [x] Close window keeps app running
- [x] Tray menu works
- [x] Quit from tray stops Ollama and exits
- [x] Ollama persists when window hidden

#### Files Created/Modified
- `src-tauri/src/main.rs` (tray setup)
- `src-tauri/tauri.conf.json` (tray config)
- `src-tauri/src/tray.rs` (tray module)

---

### Phase 6: Testing & Debugging Framework
**Duration Estimate:** 3-4 hours  
**Dependencies:** All previous phases  
**Goal:** Comprehensive test suite and debugging tools

#### Checklist
- [x] Write unit tests for config merge logic
- [x] Write unit tests for model profile selection
- [x] Write unit tests for path utilities
- [x] Set up logging with `log` crate and `fern`
- [x] Implement log file rotation
- [x] Create debug panel (troubleshooting mode)
- [x] Add manual E2E test checklist
- [x] Document all test commands

#### Exit Criteria
- [ ] `cargo test` passes all tests
- [ ] Logs write to correct location
- [ ] Debug panel accessible via hidden trigger
- [ ] E2E checklist covers all user flows

#### Files Created/Modified
- `src-tauri/src/tests/` (test modules)
- `src-tauri/src/utils/logging.rs`
- `src/lib/components/DebugPanel.svelte`
- `TESTING.md` (E2E checklist)

---

### Phase 7: Build & Package
**Duration Estimate:** 2-3 hours  
**Dependencies:** All previous phases  
**Goal:** Production builds for Windows and macOS

#### Checklist
- [ ] Configure Tauri bundle settings (msi, dmg)
- [ ] Build Windows x64 installer
- [ ] Build macOS Intel installer
- [ ] Build macOS ARM installer
- [ ] Create universal macOS binary (optional)
- [ ] Test installers on clean VMs
- [ ] Set up GitHub Actions CI/CD (optional)
- [ ] Configure auto-updater (optional)

#### Exit Criteria
- [ ] MSI installs and runs on Windows
- [ ] DMG mounts and runs on macOS Intel
- [ ] DMG mounts and runs on macOS ARM
- [ ] No antivirus false positives
- [ ] Smoke tests pass on each platform

#### Files Created/Modified
- `src-tauri/tauri.conf.json` (bundle config)
- `.github/workflows/build.yml` (CI/CD)
- `build/` (output directory)

---

## Errors Encountered Log

| Phase | Error | Attempt | Resolution | Status |
|-------|-------|---------|------------|--------|
| | | | | |

---

## Decisions Made

| Decision | Rationale | Date |
|----------|-----------|------|
| Implement Ollama model listing/download via HTTP API before installer work | Matches the requirements to use Ollama's REST API directly and unblocks later wizard/dashboard phases that need model visibility | 2026-03-18 |
| Refactor to a shared retrying HTTP client before installer work | Consolidates version/model/pull network behavior and reduces duplicated reqwest setup before adding more Ollama API calls | 2026-03-18 |
| Use Ollama's current official download assets (`OllamaSetup.exe`, `Ollama.dmg`) instead of the older `.tgz` assumption | Aligns the backend with the current Ollama download pages and keeps platform installer logic accurate to the current distribution format | 2026-03-18 |
| Preserve unknown Handy config JSON with `serde(flatten)` before expanding config write paths | Prevents the launcher from silently deleting unrelated Handy settings when it rewrites `settings_store.json` during setup | 2026-03-18 |
| Switch Handy configuration writes to raw JSON mutation instead of the placeholder typed provider model | The documented Handy schema uses `custom`, `base_url`, and prompt structures that may be object- or array-shaped, so configuration must patch the real JSON shape directly | 2026-03-18 |
| Use a direct `open_ollama_download_page` fallback instead of a dead-end troubleshooting placeholder in the setup wizard | Gives the incomplete installer state a real manual recovery path while keeping troubleshooting/log access in the launcher-owned data directory | 2026-03-18 |
| Defer Ollama auto-restart until after the minimal tray lifecycle is stable | Keeps Phase 5 focused on a safe tray/hide-to-background slice before introducing background restart coordination | 2026-03-18 |
| Supervise only launcher-managed Ollama processes with a generation-based runtime token | Prevents the background monitor from restarting externally managed or explicitly stopped Ollama instances while still allowing automatic recovery after launcher-started crashes | 2026-03-17 |
| Use the launcher-owned data directory for backend logs with startup rotation | Keeps diagnostics alongside retained backups/downloads and satisfies the 10 MB / 5-file retention requirement without introducing a separate storage location | 2026-03-17 |
| | | |

---

## Session History Summary

| Date | Phase | Work Done | Status |
|------|-------|-----------|--------|
| 2026-03-17 | 0 | Created implementation plan and documentation | In Progress |
| 2026-03-17 | 5 | Added managed Ollama background monitoring and auto-restart supervision | Completed |
| 2026-03-17 | 6 | Added backend logging, test command documentation, and path/log rotation coverage | In Progress |
| 2026-03-17 | 6 | Added hidden-trigger debug panel with live log tail and raw state view | In Progress |

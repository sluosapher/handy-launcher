# Findings: Handy Launcher Documentation

## Existing Documentation Review

### Requirements Analysis (docs/requirements-analysis.md)
**Status:** Complete ?  
**Coverage:**
- Product vision and target market
- User personas (Privacy-Focused Professional, Technical Minimalist)
- Functional requirements (auto-download, config merge, process management)
- Non-functional requirements (performance, security, usability)
- Hardware requirements (CPU, RAM, disk space)

**Gaps for Implementation:**
- No specific Ollama version requirements
- No detailed error scenarios and recovery flows
- No API contract specifications

### System Architecture (docs/architecture/system-architecture.md)
**Status:** Complete ?  
**Coverage:**
- Technology stack (Tauri, Svelte, Rust)
- High-level component diagram
- Backend module responsibilities (Ollama Manager, Handy Config Manager, State Store)
- Data flow between layers

**Gaps for Implementation:**
- No specific Tauri command signatures
- No data structure definitions
- No error handling patterns specified
- No state machine for setup wizard flow

## Technical Decisions Log

### Decision: Ollama Installation Location
**Context:** Need to install Ollama without admin privileges  
**Decision:** User data directory (`%APPDATA%/HandyLauncher/ollama` on Windows, `~/Library/Application Support/HandyLauncher/ollama` on macOS)  
**Rationale:** 
- No admin rights required
- Self-contained, easy to uninstall
- Follows Tauri app data directory conventions

### Decision: Configuration Merge Strategy
**Context:** Handy uses settings.json that must be modified to enable Ollama  
**Decision:** JSON merge with backup/restore capability  
**Rationale:**
- Non-destructive (backup before modify)
- Allows user to revert
- Handles nested object merging

### Decision: Process Management Approach
**Context:** Ollama needs to run as a background process  
**Decision:** App-managed lifecycle (start on setup, stop on app exit)  
**Rationale:**
- User doesn't need to know Ollama is running
- Clean shutdown when app closes
- Can monitor health and restart if needed

## Open Questions

1. **Model Download Strategy:** Should we download models during setup or on first use?
   - During setup: longer initial setup, guaranteed ready
   - On first use: faster setup, potential delay on first transcription

2. **Ollama Version Pinning:** Should we pin to a specific Ollama version or use latest?
   - Pinning: more predictable, requires app updates for Ollama updates
   - Latest: automatic improvements, potential breaking changes

3. **Error Recovery:** What happens if Ollama download fails mid-way?
   - Resume partial download?
   - Start over?
   - Manual fallback instructions?

## Research Notes

### Ollama API Endpoints Required
Based on Ollama documentation:
- `GET /api/tags` - List downloaded models
- `POST /api/pull` - Download a model
- `POST /api/generate` - Run inference (for testing)
- `GET /api/version` - Check Ollama version

### Tauri Commands Needed
- `check_ollama_status()` -> OllamaStatus
- `install_ollama()` -> Result<InstallProgress, InstallError>
- `download_model(model_name: String)` -> Result<DownloadProgress, DownloadError>
- `configure_handy(model_name: String)` -> Result<(), ConfigError>
- `get_system_info()` -> SystemInfo (RAM, CPU, disk space)

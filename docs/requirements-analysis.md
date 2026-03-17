# Requirements Analysis Document
## Handy Launcher

**Version:** 1.0  
**Date:** March 16, 2026  
**Status:** Draft

---

## 1. Executive Summary

Handy Launcher is a companion application that simplifies the setup of local Large Language Models (LLMs) for post-processing voice-to-text transcriptions in the Handy voice application. It automates model download, configuration, and integration to provide a seamless local AI experience without requiring technical expertise.

---

## 2. Product Vision

### 2.1 Vision Statement
Make local LLM-powered transcription post-processing accessible to any Handy user within 5 minutes of installation, with zero command-line interaction.

### 2.2 Target Market
- Handy voice-to-text application users
- Privacy-conscious users preferring local AI
- Users without technical background to set up LLMs manually

---

## 3. User Personas

### 3.1 Primary Persona: Privacy-Focused Professional
- **Name:** Sarah Chen
- **Role:** UX Designer
- **Context:** Uses Handy for transcribing meeting notes and design thoughts
- **Pain Points:**
  - Wants to keep voice data private (no cloud LLMs)
  - Intimidated by downloading models, configuring APIs, and setting environment variables
  - Previously tried to set up local models but gave up after 2 hours of troubleshooting
- **Goals:**
  - Click one button to enable local post-processing
  - Have transcription quality comparable to cloud providers
  - Understand if her hardware can handle it

### 3.2 Secondary Persona: Technical Minimalist
- **Name:** Michael Rodriguez
- **Role:** Software Developer
- **Context:** Comfortable with technology but values convenience
- **Pain Points:**
  - Knows Ollama exists but doesn't want to manage it manually
  - Wants to recommend Handy to non-technical family members
- **Goals:**
  - Quick, reproducible setup across multiple machines
  - Ability to verify what the launcher does under the hood

---

## 4. Requirements

### 4.1 Functional Requirements

#### FR-1: Ollama Management
**User Story:** As a user, I want Handy Launcher to automatically install and manage Ollama so that I don't need to use the command line.

**Acceptance Criteria:**
- WHEN the launcher starts and detects Ollama is not installed THEN it SHALL display an installation button
- WHEN the user clicks install THEN the launcher SHALL download and install Ollama silently
- WHEN installation completes THEN the launcher SHALL verify Ollama functionality
- WHEN Ollama is already installed THEN the launcher SHALL skip installation and proceed
- WHEN on Windows THEN the launcher SHALL use Ollama's Windows installer
- WHEN on macOS THEN the launcher SHALL use Ollama's macOS package (Apple Silicon/Intel detection)

#### FR-2: Model Download and Selection
**User Story:** As a user, I want to choose from pre-configured model profiles so that I can pick the right balance of speed and quality for my hardware.

**Acceptance Criteria:**
- WHEN the user reaches the model selection screen THEN the launcher SHALL display three profiles: Light, Fast, and Balanced
- WHEN Light is selected THEN the launcher SHALL configure model `llama3.2:1b`
- WHEN Fast is selected THEN the launcher SHALL configure model `phi4-mini`
- WHEN Balanced is selected THEN the launcher SHALL configure model `qwen2.5:7b`
- WHEN a profile is selected THEN the launcher SHALL display:
  - Approximate download size
  - Estimated RAM/VRAM requirements
  - Expected transcription speed (tokens/sec)
- WHEN the user confirms THEN the launcher SHALL download the model via Ollama API
- WHEN download is in progress THEN the launcher SHALL show progress bar with pause/cancel options
- WHEN download fails THEN the launcher SHALL display error and retry option with alternative profile suggestions

#### FR-3: Handy Configuration Integration
**User Story:** As a user, I want Handy Launcher to automatically configure Handy's settings so that post-processing works immediately.

**Acceptance Criteria:**
- WHEN model setup completes THEN the launcher SHALL modify Handy's settings file at:
  - Windows: `%APPDATA%/com.pais.handy/settings_store.json`
  - macOS: `~/Library/Application Support/com.pais.handy/settings_store.json`
- WHEN configuring THEN the launcher SHALL set:
  - `post_process_provider_id`: `"custom"`
  - `post_process_providers[custom].base_url`: `"http://127.0.0.1:63452/v1"`
  - `post_process_models.custom`: selected model name
  - `post_process_selected_prompt_id`: `"handy_launcher_optimized"`
  - `post_process_prompts`: append user's optimized prompt with ID `"handy_launcher_optimized"`
- WHEN configuration is written THEN the launcher SHALL backup existing configuration
- WHEN configuration fails THEN the launcher SHALL restore backup and display error

#### FR-4: Launcher Persistence and Monitoring
**User Story:** As a user, I want the launcher to keep Ollama running and optionally launch Handy so that everything works together seamlessly.

**Acceptance Criteria:**
- WHEN configuration completes THEN the launcher SHALL start Ollama server on port 63452
- WHEN Ollama starts THEN the launcher SHALL verify `/api/tags` endpoint responds within 30 seconds
- WHEN Ollama is running THEN the launcher SHALL display:
  - Current model status (loaded/unloaded)
  - Server health indicator
  - Memory usage (optional for MVP)
- WHEN the user clicks "Launch Handy" THEN the launcher SHALL check if Handy is installed
- WHEN Handy is installed THEN the launcher SHALL launch Handy application
- WHEN Handy is not found THEN the launcher SHALL prompt user to download from pa.is/handy
- WHEN the launcher window closes (X button) THEN the launcher SHALL minimize to system tray
- WHEN exit is selected from tray menu THEN the launcher SHALL:
  - Stop Ollama (graceful shutdown)
  - Exit within 5 seconds

#### FR-5: Hardware Compatibility Check
**User Story:** As a user, I want to know if my hardware supports local model inference so that I can set realistic expectations.

**Acceptance Criteria:**
- WHEN the launcher starts THEN it SHALL check:
  - Available RAM (>8GB recommended)
  - Available disk space (>10GB recommended)
  - GPU availability (CUDA on Windows, Metal on macOS)
- WHEN hardware is insufficient THEN the launcher SHALL display warning with specific issues
- WHEN GPU is unavailable THEN the launcher SHALL note CPU-only mode with slower performance expectation
- WHEN hardware check passes THEN the launcher SHALL display "Your system is ready" message

---

### 4.2 Non-Functional Requirements

#### NFR-1: Performance
- Model download SHALL support resume after interruption
- UI SHALL remain responsive during background operations
- Startup time SHALL be <3 seconds (launcher only, model loading separate)

#### NFR-2: Reliability
- Configuration changes SHALL be atomic (backup before write)
- Ollama process SHALL restart automatically if crashed (max 3 attempts)
- Duplicate Ollama instances SHALL be detected and managed

#### NFR-3: Security
- User configuration backup SHALL be saved in user data directory
- No sensitive information SHALL be logged to disk
- API keys (even for dummy local endpoints) SHALL be handled securely per Handy's format

#### NFR-4: Usability
- Interface SHALL use clear, non-technical language
- Error messages SHALL include actionable next steps
- Help documentation SHALL be accessible within 2 clicks

#### NFR-5: Compatibility
- Windows 10/11 (64-bit) SHALL be supported
- macOS 12+ ARM and Intel SHALL be supported
- Launcher SHALL detect and respect Handy installation method (Store vs direct)

---

## 5. Out of Scope (MVP)

The following features are explicitly OUT OF SCOPE for V1 but may be considered for future releases:

- Custom model downloads (beyond the 3 curated profiles)
- Prompt customization UI (user can view and reset, but not create custom prompts)
- Multiple model management (simultaneous loaded models)
- Remote Ollama instances (other host:port configurations)
- GPU selection (when multiple GPUs present)
- Linux support
- Configuration import/export
- Usage analytics/statistics
- Model fine-tuning or customization
- Alternative backends (llama.cpp, LM Studio, KoboldCPP, etc.)
- Configuration backup/restore (backups are kept but no user-facing restore UI)
## 6. Success Metrics

| Metric | Target |
|--------|--------|
| Setup completion rate | >85% of users who start complete within 10 min |
| Post-processing success rate | >95% of configured setups work on first Handy use |
| User-reported friction | <10% of users report setup difficulty in feedback |
| Average setup time | <5 minutes (excluding model download) |

---

## 7. Technical Implementation Requirements

### 7.1 Ollama Integration Method
**Requirement:** Handy Launcher SHALL interact with Ollama using its programmatic SDK/API, NOT command-line tools.

**Rationale:**
- Command-line parsing is fragile and subject to breaking changes
- SDK provides structured error handling and type safety
- Better performance through direct API calls vs process spawning
- Easier to detect and handle edge cases

**Acceptance Criteria:**
- WHEN checking Ollama status THEN the launcher SHALL use HTTP API to localhost:11434/api/version
- WHEN downloading models THEN the launcher SHALL use /api/pull endpoint with streaming response
- WHEN running inference tests THEN the launcher SHALL use /api/generate or /api/chat endpoints
- WHEN managing Ollama process THEN the launcher SHALL use OS process management APIs, NOT ollama serve CLI
- WHEN checking installed models THEN the launcher SHALL use /api/tags endpoint

**Prohibited:**
- Parsing output from ollama list, ollama pull, ollama run commands
- Spawning shell processes for Ollama operations
- Reading Ollama CLI stdout/stderr for status

### 7.2 Ollama SDK/Library Usage
| Platform | Recommended Approach |
|----------|---------------------|
| Windows | Direct HTTP calls to Ollama REST API |
| macOS | Direct HTTP calls to Ollama REST API |

**Note:** Ollama exposes a REST API on localhost:11434. The launcher should use HTTP client libraries (e.g., etch, xios, eqwest, or platform-native HTTP clients) rather than wrapping CLI commands.

---

## 8. Dependencies and Assumptions

### Assumptions:
1. Handy application continues to use the current `settings_store.json` format
2. Handy supports custom providers with configurable `base_url`
3. Ollama maintains stable OpenAI-compatible API format
4. Selected models remain available in Ollama repository

### External Dependencies:
- Ollama runtime (automatically installed by launcher)
- Handy application (pre-installed or downloaded separately)

---

## 9. Open Questions

| Question | Priority | Status | Notes |
|----------|----------|--------|-------|
| Should launcher auto-update Ollama when new versions release? | Medium | Open | Maintenance burden |
| Should launcher support "pause" feature to unload model from memory? | Low | Open | Memory management |
| Should we include automatic Handy download if not found? | Medium | Open | Integration flow |
| Should we support custom user prompts beyond the default? | Low | Open | V2 feature candidate |

**Resolved Questions:**
- ~~What's the optimal default prompt for transcription post-processing?~~ - User provides optimized prompt
- ~~How to handle Handy running during config write?~~ - Require user to close Handy first
- ~~What happens on uninstall?~~ - Leave Handy config unchanged, notify user
## 10. Document History

| Version | Date | Author | Changes | Reviewer |
|---------|------|--------|---------|----------|
| 1.2 | 2026-03-16 | PM Team | Added Section 7: Technical Implementation Requirements specifying Ollama SDK/API usage over CLI | Pending |
| 1.1 | 2026-03-16 | PM Team | Fixed corrupted text, added FR-6 through FR-14 based on user feedback: Handy running detection, existing config handling, port conflict resolution, model verification, first/subsequent run behavior, logging, test connection, prompt management, uninstall behavior | Pending |
| 1.0 | 2026-03-16 | PM Team | Initial draft with core requirements FR-1 through FR-5 | Pending |

### Change Log Details

**Version 1.2 Changes:**
- Added explicit requirement to use Ollama REST API/SDK instead of CLI tools
- Documented prohibited CLI commands (ollama list, ollama pull, ollama run)
- Specified HTTP endpoints for all Ollama operations
- Added rationale for SDK approach (reliability, error handling, performance)

**Version 1.1 Changes:**
- Fixed encoding issue in Windows config path (was showing garbled characters)
- Reordered user flow: Ollama server starts BEFORE Handy configuration (was reversed)
- Split FR-4 into two separate requirements (Ollama management vs Handy config)
- Added FR-6: Handy Running Detection
- Added FR-7: Existing Configuration Handling  
- Added FR-8: Port Conflict Resolution
- Added FR-9: Model Verification and Testing
- Added FR-10: First Run vs Subsequent Run Behavior
- Added FR-11: Logging and Diagnostics
- Added FR-12: Test Connection Feature
- Added FR-13: Prompt Management
- Added FR-14: Uninstall Behavior
- Updated Out of Scope section with configuration backup/restore UI
- Updated Open Questions with resolved items

**Version 1.0 Changes:**
- Initial document creation
- Defined product vision and user personas
- Established core functional requirements FR-1 through FR-5
- Created user flow diagram
- Documented Handy configuration mapping
## Appendix A: Handy Configuration Mapping

### Config File Paths

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%/com.pais.handy/settings_store.json` |
| macOS | `~/Library/Application Support/com.pais.handy/settings_store.json` |

### Required Configuration Changes

```json
{
  "settings": {
    "post_process_provider_id": "custom",
    "post_process_providers": [
      {
        "allow_base_url_edit": true,
        "base_url": "http://127.0.0.1:63452/v1",
        "id": "custom",
        "label": "Custom",
        "models_endpoint": "/models",
        "supports_structured_output": false
      }
    ],
    "post_process_models": {
      "custom": "{{SELECTED_MODEL}}"
    },
    "post_process_prompts": [
      /* Appended to existing array */
      {
        "id": "handy_launcher_optimized",
        "name": "Handy Launcher Optimized",
        "prompt": "{{USER_PROMPT}}"
      }
    ],
    "post_process_selected_prompt_id": "handy_launcher_optimized"
  }
}
```

### Model Mapping

| Profile | Ollama Model | Size (Approx) | RAM Required |
|---------|--------------|---------------|--------------|
| Light | `llama3.2:1b` | ~1.0 GB | 4GB+ |
| Fast | `phi4-mini` | ~2.5 GB | 6GB+ |
| Balanced | `qwen2.5:7b` | ~4.5 GB | 8GB+ |

---


## Appendix B: User Flow Diagram

```
                                                      
    +-----------------------------+                    
    |     Start Handy Launcher    |                    
    +-------------+---------------+                     
                  |                                   
                  v                                   
    +-----------------------------+                  
    |      Hardware Check         |                  
    +-------------+---------------+                   
                  |                                   
                  v                                   
    +-----------------------------+                  
    |    Ollama Installed?        |                  
    +-------------+---------------+                   
                  |                                   
          +-------+--------+                          
          |                |                          
          v                v                          
    +-----------+    +----------------------+       
    | Install   |    |  Model Management     |       
    |  Ollama   |    |  - Select profile     |       
    | (guided  |    |  - Download model     |       
    | download) |    +----------+------------+       
    +------+----+               |                    
           |                    v                    
           |         +----------------------+         
           |         |   Start Ollama       |         
           +-------->|   Server (port       |         
                     |   63452)             |         
                     |   - Verify running   |         
                     |   - Confirm port     |         
                     +----------+-----------+         
                                |                     
                                v                     
                     +---------------------+          
                     |   Configure Handy   |          
                     |   - Write config    |          
                     |   - Backup existing |          
                     +----------+----------+          
                                |                      
                                v                      
                     +---------------------+          
                     |   Launch Handy      |          
                     |   (optional)        |          
                     +----------+----------+          
                                |                      
                                v                      
                     +---------------------+          
                     |   Monitor Status    |          
                     |   (system tray)     |          
                     +---------------------+          
                                                      

#### FR-6: Handy Running Detection
**User Story:** As a user, I want the launcher to detect if Handy is running before modifying its configuration so that changes are applied safely.

**Acceptance Criteria:**
- WHEN attempting to write Handy configuration THEN the launcher SHALL check if Handy process is running
- WHEN Handy is detected running THEN the launcher SHALL display a modal dialog requesting user to close Handy
- WHEN user confirms Handy is closed THEN the launcher SHALL retry configuration write
- WHEN user chooses to continue anyway THEN the launcher SHALL warn that changes require Handy restart

#### FR-7: Existing Configuration Handling
**User Story:** As a returning user, I want the launcher to reuse my previous configuration so that I don't need to re-download models.

**Acceptance Criteria:**
- WHEN launcher starts AND previous valid configuration exists THEN the launcher SHALL offer "Use Existing Setup" option
- WHEN user selects existing setup THEN the launcher SHALL:
  - Skip model download (if model already present)
  - Start Ollama with existing configuration
  - Verify model is available and functional
- WHEN existing model is not found THEN the launcher SHALL fall back to model download flow
- WHEN user selects "Fresh Setup" THEN the launcher SHALL proceed with full setup wizard

#### FR-8: Port Conflict Resolution
**User Story:** As a user, I want the launcher to handle port conflicts automatically so that setup succeeds even if port 63452 is unavailable.

**Acceptance Criteria:**
- WHEN starting Ollama THEN the launcher SHALL attempt port 63452 first
- WHEN port 63452 is unavailable THEN the launcher SHALL automatically find next available port (63453, 63454, etc.)
- WHEN alternative port is selected THEN the launcher SHALL:
  - Display the actual port being used
  - Configure Handy with the correct port in base_url
- WHEN no ports in range 63452-63462 are available THEN the launcher SHALL display error with instructions to free up ports

#### FR-9: Model Verification and Testing
**User Story:** As a user, I want the launcher to verify the downloaded model works before completing setup so that I know the system is functional.

**Acceptance Criteria:**
- WHEN model download completes THEN the launcher SHALL run a test inference
- WHEN test inference succeeds THEN the launcher SHALL display "Model Ready" status
- WHEN test inference fails THEN the launcher SHALL:
  - Display error with failure reason
  - Offer to retry download
  - Offer to switch to alternative model profile
- WHEN switching to alternative model THEN the launcher SHALL automatically download and test the new model

#### FR-10: First Run vs Subsequent Run Behavior
**User Story:** As a user, I want different experiences on first launch versus returning to the launcher so that I can quickly access status or reconfigure as needed.

**Acceptance Criteria:**
- WHEN launcher starts with no previous configuration THEN the launcher SHALL display Setup Wizard
- WHEN launcher starts with valid existing configuration THEN the launcher SHALL display Status Dashboard
- WHEN on Status Dashboard THEN the user SHALL be able to:
  - View current model status (running/stopped)
  - Switch to different model profile
  - View Ollama logs
  - Reset configuration to defaults
  - Check for Ollama updates
- WHEN user selects "Reconfigure" THEN the launcher SHALL return to Setup Wizard

#### FR-11: Logging and Diagnostics
**User Story:** As a user, I want access to logs when troubleshooting so that I can get help or understand issues.

**Acceptance Criteria:**
- WHEN launcher operates THEN it SHALL write logs to:
  - Windows: %APPDATA%/HandyLauncher/logs/launcher.log
  - macOS: ~/Library/Logs/HandyLauncher/launcher.log
- WHEN log file exceeds 10MB THEN the launcher SHALL rotate logs (keep last 5 files)
- WHEN user clicks "View Logs" THEN the launcher SHALL open log viewer or log directory
- WHEN user clicks "Export Logs" THEN the launcher SHALL create zip of logs for sharing
- WHEN error occurs THEN the launcher SHALL log at ERROR level with stack trace

#### FR-12: Test Connection Feature
**User Story:** As a user, I want to verify the connection to Ollama is working so that I can confirm post-processing will function.

**Acceptance Criteria:**
- WHEN Ollama is running THEN the launcher SHALL display "Test Connection" button
- WHEN user clicks "Test Connection" THEN the launcher SHALL:
  - Send test request to Ollama API
  - Display response time
  - Show model loaded status
- WHEN test succeeds THEN the launcher SHALL display green checkmark with "Connected"
- WHEN test fails THEN the launcher SHALL display error details and troubleshooting steps

#### FR-13: Prompt Management
**User Story:** As a user, I want to view and reset the post-processing prompt so that I understand what transformations are being applied.

**Acceptance Criteria:**
- WHEN in Status Dashboard THEN the user SHALL be able to view current prompt
- WHEN user modifies prompt THEN the launcher SHALL validate prompt contains required placeholder
- WHEN user clicks "Reset to Default" THEN the launcher SHALL restore original optimized prompt
- WHEN prompt is changed THEN the launcher SHALL update Handy configuration immediately

#### FR-14: Uninstall Behavior
**User Story:** As a user, when I uninstall Handy Launcher, I want to understand what happens to my Handy configuration.

**Acceptance Criteria:**
- WHEN user uninstalls Handy Launcher THEN the launcher SHALL display notification:
  - "Your Handy configuration will remain unchanged"
  - "To disable local post-processing, change Handy's provider setting"
- WHEN uninstalling THEN the launcher SHALL NOT remove or modify Handy's settings_store.json
- WHEN uninstalling THEN the launcher SHALL offer option to remove downloaded models to free disk space

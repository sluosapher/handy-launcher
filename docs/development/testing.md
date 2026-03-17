# Testing Strategy Document
## Handy Launcher

**Version:** 1.0  
**Date:** March 16, 2026  
**Status:** Draft

---

## 1. Overview

### 1.1 Testing Philosophy

Handy Launcher requires a multi-layered testing approach:

- **Unit tests** for business logic (Rust backend)
- **Integration tests** for Tauri command interfaces
- **End-to-end tests** for critical user workflows
- **Manual testing** for platform-specific behaviors and edge cases

### 1.2 Test Pyramid

```
        /\
       /  \     E2E Tests (Critical paths)
      /____\    
     /      \   Integration Tests (Tauri commands)
    /________\  
   /          \ Unit Tests (Rust modules)
  /____________\
```

**Distribution:**
- 70% Unit tests (fast, isolated)
- 20% Integration tests (command layer)
- 10% E2E tests (happy paths + critical errors)

---

## 2. Unit Tests (Rust Backend)

### 2.1 Test Organization

```
src/
u251cu2500u2500 managers/
u2502   u251cu2500u2500 ollama_manager.rs
u2502   u251cu2500u2500 config_manager.rs
u2502   u2514u2500u2500 mod.rs
u251cu2500u2500 models/
u2502   u251cu2500u2500 state.rs
u2502   u251cu2500u2500 config.rs
u2502   u2514u2500u2500 mod.rs
u2514u2500u2500 utils/
    u251cu2500u2500 paths.rs
    u2514u2500u2500 mod.rs

# Tests mirror src structure
tests/
u251cu2500u2500 unit/
u2502   u251cu2500u2500 managers/
u2502   u2502   u251cu2500u2500 ollama_manager_test.rs
u2502   u2502   u2514u2500u2500 config_manager_test.rs
u2502   u251cu2500u2500 models/
u2502   u2502   u2514u2500u2500 state_test.rs
u2502   u2514u2500u2500 utils/
u2502       u2514u2500u2500 paths_test.rs
u251cu2500u2500 integration/
u2502   u2514u2500u2500 commands_test.rs
u2514u2500u2500 fixtures/
    u251cu2500u2500 mock_ollama.rs
    u2514u2500u2500 sample_configs/
```

### 2.2 Running Unit Tests

```bash
# Run all Rust tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module
cargo test managers::

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### 2.3 Key Test Areas

#### Config Manager Tests

```rust
// tests/unit/managers/config_manager_test.rs

#[cfg(test)]
mod config_manager_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_locate_handy_config_windows() {
        // Mock APPDATA environment
        env::set_var("APPDATA", "C:\\Users\\Test\\AppData\\Roaming");
        
        let path = ConfigManager::locate_handy_config();
        
        assert!(path.is_ok());
        assert!(path.unwrap().to_string_lossy().contains("com.pais.handy"));
    }

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("settings_store.json");
        
        // Create test config
        fs::write(&config_path, r#"{"test": "value"}"#).unwrap();
        
        let manager = ConfigManager::new(config_path.clone());
        let backup_path = manager.create_backup().unwrap();
        
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backup"));
        assert!(backup_path.extension().unwrap() == "json");
    }

    #[test]
    fn test_merge_preserves_existing_keys() {
        let existing = json!({
            "user_setting": "keep_this",
            "llm": { "existing": "value" }
        });
        
        let ollama_config = json!({
            "llm": { "provider": "custom" }
        });
        
        let merged = ConfigManager::merge_configs(&existing, &ollama_config);
        
        assert_eq!(merged["user_setting"], "keep_this");
        assert_eq!(merged["llm"]["existing"], "value");
        assert_eq!(merged["llm"]["provider"], "custom");
    }

    #[test]
    fn test_validate_json_structure() {
        let valid = r#"{"llm": {"provider": "custom"}}"#;
        let invalid = r#"{"llm": {"provider": }}"#;
        
        assert!(ConfigManager::validate_json(valid).is_ok());
        assert!(ConfigManager::validate_json(invalid).is_err());
    }

    #[test]
    fn test_restore_backup_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("settings_store.json");
        let original_content = r#"{"original": "data"}"#;
        
        fs::write(&config_path, original_content).unwrap();
        
        // Simulate failed write
        let manager = ConfigManager::new(config_path.clone());
        let result = manager.write_config_with_rollback(|| {
            Err("Simulated write failure".into())
        });
        
        assert!(result.is_err());
        
        // Verify original restored
        let restored = fs::read_to_string(&config_path).unwrap();
        assert_eq!(restored, original_content);
    }
}
```

#### Ollama Manager Tests

```rust
// tests/unit/managers/ollama_manager_test.rs

#[cfg(test)]
mod ollama_manager_tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_list_models_success() {
        let _m = mock("GET", "/api/tags")
            .with_status(200)
            .with_body(r#"{
                "models": [
                    {"name": "llama3.2:1b", "size": 1300000000}
                ]
            }"#)
            .create();

        let client = OllamaClient::new(11434);
        let models = client.list_models().await.unwrap();
        
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "llama3.2:1b");
    }

    #[tokio::test]
    async fn test_list_models_connection_refused() {
        // Test with no server running
        let client = OllamaClient::new(99999); // Invalid port
        let result = client.list_models().await;
        
        assert!(matches!(result, Err(LauncherError::ConnectionFailed(_))));
    }

    #[tokio::test]
    async fn test_port_discovery() {
        let manager = OllamaManager::new();
        
        // Mock port 63452 as unavailable
        // Implementation would bind a test socket
        
        let port = manager.find_available_port(63452..63462).await;
        
        assert!(port >= 63452);
        assert!(port <= 63462);
    }

    #[test]
    fn test_model_profile_mapping() {
        let profiles = ModelProfiles::default();
        
        let fast = profiles.get("fast").unwrap();
        assert_eq!(fast.model_name, "llama3.2:1b");
        assert_eq!(fast.ram_required_gb, 2);
        
        let recommended = profiles.get("recommended").unwrap();
        assert_eq!(recommended.model_name, "phi4:mini");
    }

    #[tokio::test]
    async fn test_health_check_timeout() {
        let client = OllamaClient::new(11434);
        
        // Mock slow response
        let _m = mock("GET", "/api/version")
            .with_status(200)
            .with_body(r#"{"version": "0.6.7"}"#)
            .with_delay(std::time::Duration::from_secs(10))
            .create();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.wait_for_healthy(3)
        ).await;
        
        assert!(result.is_err() || result.unwrap().is_err());
    }
}
```

#### State Management Tests

```rust
// tests/unit/models/state_test.rs

#[cfg(test)]
mod state_tests {
    use super::*;

    #[test]
    fn test_app_state_transitions() {
        let mut state = AppState::default();
        
        assert_eq!(state.setup_status, SetupStatus::NotStarted);
        
        state.transition_to(SetupStatus::SystemCheck);
        assert_eq!(state.setup_status, SetupStatus::SystemCheck);
        
        state.transition_to(SetupStatus::Complete);
        assert_eq!(state.setup_status, SetupStatus::Complete);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let state = AppState {
            setup_status: SetupStatus::Complete,
            selected_model: Some("llama3.2:1b".to_string()),
            ollama_port: Some(63452),
        };
        
        let json = serde_json::to_string(&state).unwrap();
        let restored: AppState = serde_json::from_str(&json).unwrap();
        
        assert_eq!(state.setup_status, restored.setup_status);
        assert_eq!(state.selected_model, restored.selected_model);
    }
}
```

### 2.4 Test Fixtures

```rust
// tests/fixtures/mock_ollama.rs

pub struct MockOllamaServer {
    port: u16,
}

impl MockOllamaServer {
    pub fn new() -> Self {
        // Start mock server on random port
        Self { port: 0 }
    }
    
    pub fn mock_list_models(&self) -> Mock {
        mock("GET", "/api/tags")
            .with_status(200)
            .with_body_from_file("tests/fixtures/responses/list_models.json")
    }
    
    pub fn mock_pull_stream(&self) -> Mock {
        mock("POST", "/api/pull")
            .with_status(200)
            .with_body_from_file("tests/fixtures/responses/pull_stream.ndjson")
    }
}

// tests/fixtures/sample_configs/settings_store.json
{
  "llm": {
    "provider": "openai",
    "model": "gpt-4"
  },
  "user_preferences": {
    "theme": "dark"
  }
}
```

---

## 3. Integration Tests (Tauri Commands)

### 3.1 Test Setup

Integration tests verify Tauri commands work correctly through the invoke API.

```bash
# Run integration tests
cargo test --test integration

# With Tauri runtime
cargo test --features tauri-test
```

### 3.2 Command Tests

```rust
// tests/integration/commands_test.rs

use tauri::test::mock_builder;

#[tokio::test]
async fn test_check_system_requirements() {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![check_system_requirements])
        .build();
    
    let result = app
        .invoke("check_system_requirements", ())
        .await
        .unwrap();
    
    let info: SystemInfo = serde_json::from_value(result).unwrap();
    
    assert!(info.ram_gb > 0);
    assert!(info.disk_gb > 0);
    assert!(matches!(info.os, OsType::Windows | OsType::MacOS));
}

#[tokio::test]
async fn test_install_ollama_flow() {
    let temp_dir = TempDir::new().unwrap();
    let app = setup_test_app(temp_dir.path());
    
    // Test install command
    let result = app
        .invoke("install_ollama", InstallOptions { silent: true })
        .await;
    
    // Should fail in test environment (no actual download)
    assert!(result.is_err());
    
    // Verify error is user-friendly
    let error = result.unwrap_err();
    assert!(error.to_string().contains("download"));
}

#[tokio::test]
async fn test_configure_happy_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings_store.json");
    
    // Create existing config
    fs::write(&config_path, r#"{"existing": "value"}"#).unwrap();
    
    let app = setup_test_app_with_config(temp_dir.path());
    
    let result = app
        .invoke("configure_handy", ConfigureArgs {
            model_name: "llama3.2:1b".to_string(),
            port: 63452,
        })
        .await;
    
    assert!(result.is_ok());
    
    // Verify config merged correctly
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&config_path).unwrap()
    ).unwrap();
    
    assert_eq!(config["existing"], "value"); // Preserved
    assert_eq!(config["llm"]["provider"], "custom"); // Added
}

#[tokio::test]
async fn test_ollama_lifecycle() {
    let app = setup_test_app();
    
    // Start Ollama
    let start_result = app.invoke("start_ollama", ()).await;
    // May fail in test env, but should return proper error
    
    if start_result.is_ok() {
        // Test health check
        let health = app.invoke("check_ollama_health", ()).await.unwrap();
        let status: OllamaStatus = serde_json::from_value(health).unwrap();
        assert!(matches!(status, OllamaStatus::Running { .. }));
        
        // Stop Ollama
        let stop_result = app.invoke("stop_ollama", ()).await;
        assert!(stop_result.is_ok());
    }
}
```

### 3.3 Mock External Dependencies

```rust
// tests/helpers/mock_server.rs

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

pub struct MockOllamaApi {
    server: MockServer,
}

impl MockOllamaApi {
    pub async fn new() -> Self {
        Self {
            server: MockServer::start().await,
        }
    }
    
    pub fn url(&self) -> String {
        self.server.uri()
    }
    
    pub async fn mock_version(&self) {
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({"version": "0.6.7"})))
            .mount(&self.server)
            .await;
    }
    
    pub async fn mock_tags(&self, models: Vec<&str>) {
        let models_json: Vec<_> = models.iter()
            .map(|m| json!({"name": m, "size": 1000000}))
            .collect();
        
        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({"models": models_json})))
            .mount(&self.server)
            .await;
    }
}
```

---

## 4. End-to-End Tests

### 4.1 E2E Test Strategy

E2E tests use Tauri's WebDriver integration or manual testing for critical paths.

**Critical Paths to Test:**
1. First-time setup wizard (happy path)
2. Model download and configuration
3. Status dashboard operations
4. Error recovery flows

### 4.2 WebDriver Setup (Optional)

```bash
# Install WebDriver dependencies
cargo install tauri-driver

# Run E2E tests
npm run test:e2e
```

### 4.3 E2E Test Example

```typescript
// e2e/setup-wizard.spec.ts
import { test, expect } from "@playwright/test";
import { launchApp } from "./helpers/app";

test.describe("Setup Wizard", () => {
  let app;
  
  test.beforeEach(async () => {
    app = await launchApp({
      clearState: true, // Fresh install
    });
  });
  
  test.afterEach(async () => {
    await app.close();
  });
  
  test("completes full setup flow", async () => {
    // Step 1: Welcome
    await expect(app.locator("text=Enable Local Transcription")).toBeVisible();
    await app.click("text=Get Started");
    
    // Step 2: System Check (auto-advances)
    await expect(app.locator("text=Checking your device")).toBeVisible();
    await app.waitForSelector("text=Choose transcription quality", { timeout: 10000 });
    
    // Step 3: Quality Selection
    await expect(app.locator("text=Fast")).toBeVisible();
    await expect(app.locator("text=Recommended")).toBeVisible();
    await expect(app.locator("text=Accurate")).toBeVisible();
    
    // Select Recommended (pre-selected)
    await app.click("text=Confirm & Download");
    
    // Should close to tray, show download progress
    await app.waitForTimeout(500);
    
    // Verify tray icon exists
    const tray = await app.evaluate(() => {
      return window.__TAURI__.tray.getById("main");
    });
    expect(tray).toBeDefined();
  });
  
  test("shows unsuitable options as disabled", async () => {
    await app.click("text=Get Started");
    await app.waitForSelector("text=Choose transcription quality");
    
    // Mock low RAM system
    await app.evaluate(() => {
      window.__MOCK_SYSTEM_INFO__ = { ram_gb: 4 };
    });
    
    // Accurate should be disabled (requires 8GB)
    const accurateCard = app.locator("text=Accurate").locator("..");
    await expect(accurateCard).toHaveClass(/unsuitable/);
    await expect(accurateCard.locator("button")).toBeDisabled();
  });
  
  test("handles download cancellation", async () => {
    // Start download
    await app.click("text=Get Started");
    await app.waitForSelector("text=Choose transcription quality");
    await app.click("text=Confirm & Download");
    
    // Open dashboard
    await app.evaluate(() => {
      window.__TAURI__.window.getByLabel("dashboard").show();
    });
    
    // Cancel download
    await app.click("text=Cancel Download");
    await app.click("text=Yes, cancel"); // Confirm dialog
    
    // Should return to selection
    await expect(app.locator("text=Choose transcription quality")).toBeVisible();
  });
});
```

### 4.4 Manual E2E Test Checklist

Since full E2E automation is complex for desktop apps, maintain a manual test checklist:

**Test Matrix:**

| Test Case | Windows 10 | Windows 11 | macOS Intel | macOS Apple Silicon |
|-----------|------------|------------|-------------|---------------------|
| Fresh install wizard | u25a1 | u25a1 | u25a1 | u25a1 |
| System check pass | u25a1 | u25a1 | u25a1 | u25a1 |
| System check warning (low RAM) | u25a1 | u25a1 | u25a1 | u25a1 |
| Model download | u25a1 | u25a1 | u25a1 | u25a1 |
| Download cancellation | u25a1 | u25a1 | u25a1 | u25a1 |
| Config merge | u25a1 | u25a1 | u25a1 | u25a1 |
| Tray icon operations | u25a1 | u25a1 | u25a1 | u25a1 |
| Dashboard open/close | u25a1 | u25a1 | u25a1 | u25a1 |
| Test connection | u25a1 | u25a1 | u25a1 | u25a1 |
| Reconfigure flow | u25a1 | u25a1 | u25a1 | u25a1 |
| Dark mode | u25a1 | u25a1 | u25a1 | u25a1 |
| Error recovery | u25a1 | u25a1 | u25a1 | u25a1 |

---

## 5. Test Data

### 5.1 Mock Responses

```json
// tests/fixtures/responses/list_models.json
{
  "models": [
    {
      "name": "llama3.2:1b",
      "model": "llama3.2:1b",
      "modified_at": "2026-03-16T10:30:00Z",
      "size": 1300000000,
      "digest": "sha256:abc123",
      "details": {
        "format": "gguf",
        "family": "llama",
        "parameter_size": "1B",
        "quantization_level": "Q4_0"
      }
    },
    {
      "name": "phi4:mini",
      "model": "phi4:mini",
      "modified_at": "2026-03-16T10:30:00Z",
      "size": 3800000000,
      "digest": "sha256:def456",
      "details": {
        "format": "gguf",
        "family": "phi",
        "parameter_size": "4B",
        "quantization_level": "Q4_K_M"
      }
    }
  ]
}

// tests/fixtures/responses/pull_stream.ndjson
{"status": "downloading", "completed": 0, "total": 1300000000}
{"status": "downloading", "completed": 650000000, "total": 1300000000}
{"status": "success"}

// tests/fixtures/responses/version.json
{"version": "0.6.7"}
```

### 5.2 Sample Configs

```json
// tests/fixtures/sample_configs/minimal.json
{}

// tests/fixtures/sample_configs/with_llm.json
{
  "llm": {
    "provider": "openai",
    "api_key": "sk-test",
    "model": "gpt-4"
  }
}

// tests/fixtures/sample_configs/corrupted.json
{
  "llm": {
    "provider": 
  }
}

// tests/fixtures/sample_configs/large.json
{
  "user_preferences": {
    "theme": "dark",
    "language": "en",
    "shortcuts": {
      "record": "Cmd+Shift+R",
      "stop": "Cmd+Shift+S"
    }
  },
  "llm": {
    "provider": "openai",
    "model": "gpt-4"
  },
  "transcription": {
    "engine": "whisper",
    "language": "auto"
  }
}
```

---

## 6. Performance Tests

### 6.1 Benchmarks

```rust
// benches/config_merge.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_config_merge(c: &mut Criterion) {
    let existing = load_large_config();
    let ollama_config = create_ollama_config();
    
    c.bench_function("config_merge", |b| {
        b.iter(|| {
            ConfigManager::merge_configs(
                black_box(&existing),
                black_box(&ollama_config)
            )
        });
    });
}

criterion_group!(benches, bench_config_merge);
criterion_main!(benches);
```

### 6.2 Performance Criteria

| Operation | Target | Maximum |
|-----------|--------|---------|
| Config merge | < 10ms | 50ms |
| System check | < 500ms | 2s |
| Ollama health check | < 100ms | 500ms |
| UI render (wizard step) | < 16ms | 33ms |
| Window open | < 100ms | 500ms |

---

## 7. Security Tests

### 7.1 Test Cases

```rust
#[test]
fn test_config_path_traversal() {
    let malicious = "../../../etc/passwd";
    let result = ConfigManager::resolve_path(malicious);
    
    // Should sanitize or reject
    assert!(result.is_err() || !result.unwrap().to_string_lossy().contains(".."));
}

#[test]
fn test_no_secrets_in_logs() {
    let config = json!({
        "api_key": "sk-secret123",
        "password": "hunter2"
    });
    
    let log_output = format!("{:?}", config);
    
    assert!(!log_output.contains("sk-secret"));
    assert!(!log_output.contains("hunter2"));
}

#[test]
fn test_ollama_localhost_only() {
    let client = OllamaClient::new(63452);
    
    // Verify URL is localhost
    assert!(client.base_url.contains("127.0.0.1"));
    assert!(!client.base_url.contains("0.0.0.0"));
}
```

---

## 8. Continuous Integration

### 8.1 GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test-rust:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run integration tests
        run: cargo test --test integration
      
      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: "20"
      
      - name: Install dependencies
        run: |
          cd frontend
          npm ci
      
      - name: Run Svelte tests
        run: |
          cd frontend
          npm test
      
      - name: Run lint
        run: |
          cd frontend
          npm run lint
```

---

## 9. Debugging Failed Tests

### 9.1 Common Issues

| Issue | Solution |
|-------|----------|
| Port conflicts | Use `TempDir` for isolated test environments |
| Async timing | Use `tokio::time::timeout` and `retry` utilities |
| File permissions | Run tests with appropriate user, mock filesystem when possible |
| Platform differences | Use conditional compilation `#[cfg(target_os = "windows")]` |

### 9.2 Test Logging

```rust
// Enable debug logging in tests
#[cfg(test)]
mod tests {
    use env_logger;
    
    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }
    
    #[test]
    fn test_with_logging() {
        init();
        log::debug!("This will appear in test output");
    }
}
```

---

## 10. Test Checklist Before Release

### 10.1 Pre-Release Verification

- [ ] All unit tests pass (`cargo test`)
- [ ] All integration tests pass
- [ ] Manual E2E tests pass on all platforms
- [ ] No compiler warnings
- [ ] Code coverage > 80%
- [ ] Security tests pass
- [ ] Performance benchmarks meet targets
- [ ] Documentation updated

### 10.2 Platform-Specific Tests

**Windows:**
- [ ] Installer works on Windows 10/11
- [ ] System tray integration
- [ ] Path handling with spaces
- [ ] Registry not modified (user-local install)

**macOS:**
- [ ] Notarization passes
- [ ] Menubar integration
- [ ] Apple Silicon and Intel builds
- [ ] Gatekeeper allows execution

---

## 11. References

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/)
- [Mockito Documentation](https://docs.rs/mockito/latest/mockito/)
- [WireMock Rust](https://docs.rs/wiremock/latest/wiremock/)
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)

---

*Document created: March 16, 2026*

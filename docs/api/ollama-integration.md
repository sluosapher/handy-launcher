# Ollama API Integration Specification
## Handy Launcher

**Version:** 1.0  
**Date:** March 16, 2026  
**Status:** Draft

---

## 1. Overview

This document specifies how Handy Launcher integrates with the Ollama HTTP API. Ollama provides **two API modes**:

1. **Native Ollama API** — For model management (list, pull, delete)
2. **OpenAI-compatible API** — For chat completions (used by Handy for post-processing)

Handy Launcher configures Ollama to expose both endpoints, but Handy (the voice app) primarily uses the OpenAI-compatible endpoint for transcription post-processing.

---

## 2. API Base URL

After Ollama starts, the base URL is:

```
http://127.0.0.1:{PORT}/api          # Native Ollama API
http://127.0.0.1:{PORT}/v1           # OpenAI-compatible API
```

Where `{PORT}` is auto-discovered in range **63452-63462** (first available).

### 2.1 URL Construction (Rust)

```rust
// src/managers/ollama_manager.rs

pub struct OllamaClient {
    base_url: String,
    port: u16,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{}", port),
            port,
            client: reqwest::Client::new(),
        }
    }
    
    /// Native Ollama API endpoint
    fn native_url(&self, path: &str) -> String {
        format!("{}/api/{}", self.base_url, path)
    }
    
    /// OpenAI-compatible endpoint
    fn openai_url(&self, path: &str) -> String {
        format!("{}/v1/{}", self.base_url, path)
    }
}
```

---

## 3. Native Ollama API Endpoints

Used by Handy Launcher for model management and health checks.

### 3.1 GET /api/tags

List all downloaded models.

**Request:**
```http
GET http://127.0.0.1:63452/api/tags
```

**Response (200 OK):**
```json
{
  "models": [
    {
      "name": "llama3.2:3b",
      "model": "llama3.2:3b",
      "modified_at": "2026-03-16T10:30:00Z",
      "size": 2019483648,
      "digest": "sha256:a4...",
      "details": {
        "format": "gguf",
        "family": "llama",
        "families": ["llama"],
        "parameter_size": "3.2B",
        "quantization_level": "Q4_K_M"
      }
    }
  ]
}
```

**Rust Implementation:**
```rust
#[derive(Debug, Deserialize)]
pub struct ModelList {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    pub details: ModelDetails,
}

pub async fn list_models(&self) -> Result<Vec<ModelInfo>, LauncherError> {
    let url = self.native_url("tags");
    let response = self.client.get(&url).send().await?;
    
    match response.status() {
        StatusCode::OK => {
            let list: ModelList = response.json().await?;
            Ok(list.models)
        }
        _ => Err(LauncherError::OllamaApiError(response.text().await?)),
    }
}
```

### 3.2 POST /api/pull

Download a model with streaming progress updates.

**Request:**
```http
POST http://127.0.0.1:63452/api/pull
Content-Type: application/json

{
  "name": "llama3.2:3b",
  "stream": true
}
```

**Response (Streaming NDJSON):**
```ndjson
{"status": "downloading", "completed": 0, "total": 2019483648}
{"status": "downloading", "completed": 104857600, "total": 2019483648}
...
{"status": "success"}
```

**Rust Implementation:**
```rust
use futures::StreamExt;

#[derive(Debug, Deserialize)]
pub struct PullProgress {
    pub status: String,
    pub completed: Option<u64>,
    pub total: Option<u64>,
}

pub async fn pull_model(
    &self,
    model_name: &str,
    mut on_progress: impl FnMut(PullProgress),
) -> Result<(), LauncherError> {
    let url = self.native_url("pull");
    let body = json!({
        "name": model_name,
        "stream": true
    });
    
    let response = self.client
        .post(&url)
        .json(&body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(LauncherError::ModelDownloadFailed(
            response.text().await?))
    }
    
    // Stream NDJSON response
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        if let Ok(line) = String::from_utf8(bytes.to_vec()) {
            if let Ok(progress) = serde_json::from_str::<PullProgress>(&line) {
                on_progress(progress);
                
                if progress.status == "success" {
                    break;
                }
            }
        }
    }
    
    Ok(())
}
```

### 3.3 GET /api/version

Get Ollama server version.

**Request:**
```http
GET http://127.0.0.1:63452/api/version
```

**Response (200 OK):**
```json
{
  "version": "0.6.7"
}
```

**Rust Implementation:**
```rust
pub async fn get_version(&self) -> Result<String, LauncherError> {
    let url = self.native_url("version");
    let response = self.client.get(&url).send().await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json["version"].as_str().unwrap_or("unknown").to_string())
    } else {
        Err(LauncherError::OllamaNotResponding)
    }
}
```

### 3.4 POST /api/generate (Optional Testing)

Generate a completion (native Ollama format). Used for testing the model.

**Request:**
```http
POST http://127.0.0.1:63452/api/generate
Content-Type: application/json

{
  "model": "llama3.2:3b",
  "prompt": "Say hello",
  "stream": false
}
```

**Response (200 OK):**
```json
{
  "model": "llama3.2:3b",
  "created_at": "2026-03-16T10:30:00Z",
  "response": "Hello! How can I help you today?",
  "done": true,
  "total_duration": 1234567890
}
```

---

## 4. OpenAI-Compatible API

Used by **Handy voice app** for transcription post-processing. Ollama implements the OpenAI Chat Completions API.

### 4.1 POST /v1/chat/completions
ndard OpenAI chat completion endpoint.

**Request:**
```http
POST http://127.0.0.1:63452/v1/chat/completions
Content-Type: application/json

{
  "model": "llama3.2:3b",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant that fixes grammar and formatting."
    },
    {
      "role": "user",
      "content": "Fix this transcription: um hello world uh how are you"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 500,
  "stream": false
}
```

**Response (200 OK):**
```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1710585600,
  "model": "llama3.2:3b",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello, world! How are you?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 32,
    "completion_tokens": 8,
    "total_tokens": 40
  }
}
```

**Handy Configuration:**
```json
{
  "llm": {
    "provider": "openai",
    "model": "llama3.2:3b",
    "baseUrl": "http://127.0.0.1:63452/v1",
    "apiKey": "ollama",  // Required by OpenAI SDK, any non-empty value works
    "timeout": 30000
  }
}
```

**Note:** Handy uses an OpenAI-compatible client library. The `apiKey` can be any non-empty string since Ollama doesn't require authentication.

### 4.2 Rust Client Example (for Testing)

```rust
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
    pub finish_reason: String,
}

impl OllamaClient {
    pub async fn test_chat_completion(
        &self,
        model: &str,
    ) -> Result<TestResult, LauncherError> {
        let url = self.openai_url("chat/completions");
        let start = Instant::now();
        
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Say hello in one word.".to_string(),
                }
            ],
            temperature: Some(0.7),
            max_tokens: Some(50),
        };
        
        let response = self.client
            .post(&url)
            .header("Authorization", "Bearer ollama")
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let completion: ChatCompletionResponse = response.json().await?;
            let latency_ms = start.elapsed().as_millis() as u64;
            
            Ok(TestResult {
                success: true,
                latency_ms,
                response: completion.choices[0].message.content.clone(),
                error: None,
            })
        } else {
            Ok(TestResult {
                success: false,
                latency_ms: 0,
                response: String::new(),
                error: Some(response.text().await?),
            })
        }
    }
}
```

### 4.3 GET /v1/models

List available models (OpenAI-compatible format).

**Request:**
```http
GET http://127.0.0.1:63452/v1/models
Authorization: Bearer ollama
```

**Response (200 OK):**
```json
{
  "object": "list",
  "data": [
    {
      "id": "llama3.2:3b",
      "object": "model",
      "created": 1710585600,
      "owned_by": "ollama"
    }
  ]
}
```

---

## 5. Health Check Strategy

### 5.1 Startup Health Check

```rust
pub async fn wait_for_healthy(&self, timeout_secs: u64) -> Result<(), LauncherError> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    
    while start.elapsed() < timeout {
        match self.get_version().await {
            Ok(_) => return Ok(()),
            Err(_) => tokio::time::sleep(Duration::from_millis(500)).await,
        }
    }
    
    Err(LauncherError::OllamaStartFailed(
        "Health check timeout".to_string()
    ))
}
```

### 5.2 Periodic Health Check

```rust
pub async fn is_healthy(&self) -> bool {
    self.get_version().await.is_ok()
}
```

---

## 6. Error Handling

### 6.1 HTTP Status Codes

| Status | Meaning | Action |
|--------|---------|--------|
| 200 | Success | Process response |
| 404 | Model not found | Suggest `ollama pull` |
| 500 | Internal server error | Retry with backoff |
| 503 | Model loading | Wait and retry |
| Connection refused | Ollama not running | Start Ollama |

### 6.2 Error Response Schema

**Native API Error:**
```json
{
  "error": "model 'unknown-model' not found"
}
```

**OpenAI-Compatible Error:**
```json
{
  "error": {
    "message": "model 'unknown-model' not found",
    "type": "not_found_error",
    "code": "model_not_found"
  }
}
```

### 6.3 Retry Logic

```rust
pub async fn with_retry<F, Fut, T>(
    operation: F,
    max_retries: u32,
) -> Result<T, LauncherError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, LauncherError>>,
{
    let mut retries = 0;
    let mut delay = Duration::from_millis(100);
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if retries >= max_retries => return Err(e),
            Err(_) => {
                retries += 1;
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}
```

---

## 7. Configuration for Handy App

### 7.1 Minimal OpenAI-Compatible Config

```json
{
  "llm": {
    "provider": "openai",
    "model": "llama3.2:3b",
    "baseUrl": "http://127.0.0.1:63452/v1",
    "apiKey": "ollama"
  },
  "transcription": {
    "postProcess": true,
    "postProcessPrompt": "Fix grammar, punctuation, and formatting. Remove filler words like 'um' and 'uh'."
  }
}
```

### 7.2 Available Models Mapping

| Model Profile | Ollama Name | Size | VRAM Needed |
|---------------|-------------|------|-------------|
| Light | `llama3.2:1b` | 1.3 GB | 2 GB |
| Fast | `llama3.2:3b` | 2.0 GB | 4 GB |
| Balanced | `phi4:mini` | 3.8 GB | 6 GB |

### 7.3 Temperature & Sampling Guidelines

| Use Case | temperature | top_p | Recommendation |
|----------|-------------|-------|----------------|
| Grammar fix only | 0.1-0.3 | 0.9 | Deterministic, minimal creativity |
| Transcription cleanup | 0.3-0.5 | 0.9 | Some flexibility for context |
| Summarization | 0.5-0.7 | 0.95 | Balanced creativity |
| Creative rewriting | 0.7-1.0 | 1.0 | Maximum creativity |

---

## 8. Testing API Integration

### 8.1 Manual Test Commands

```bash
# Check Ollama is running
curl http://127.0.0.1:63452/api/tags

# Pull a model
curl -X POST http://127.0.0.1:63452/api/pull \
  -d '{"name": "llama3.2:3b", "stream": false}'

# Test OpenAI-compatible chat
curl http://127.0.0.1:63452/v1/chat/completions \
  -H "Authorization: Bearer ollama" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3.2:3b",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### 8.2 Integration Test Script (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ollama_integration() {
        let client = OllamaClient::new(63452);
        
        // Test health
        assert!(client.is_healthy().await);
        
        // Test model exists
        let models = client.list_models().await.unwrap();
        assert!(!models.is_empty());
        
        // Test chat completion
        let result = client.test_chat_completion("llama3.2:3b").await.unwrap();
        assert!(result.success);
        assert!(result.latency_ms < 10000); // Should respond in <10s
    }
}
```

---

## 9. References

- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Ollama OpenAI Compatibility](https://github.com/ollama/ollama/blob/main/docs/openai.md)
- [OpenAI Chat Completions API](https://platform.openai.com/docs/api-reference/chat)
- [Handy Configuration Schema](../architecture/system-architecture.md#7-configuration-schema)

---

*Document created: March 16, 2026*

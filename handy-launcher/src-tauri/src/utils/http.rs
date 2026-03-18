use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

pub struct RetryingHttpClient {
    client: Client,
    max_attempts: u8,
}

impl RetryingHttpClient {
    pub fn new(timeout: Duration, max_attempts: u8) -> Result<Self, reqwest::Error> {
        let client = Client::builder().timeout(timeout).build()?;
        Ok(Self {
            client,
            max_attempts: max_attempts.max(1),
        })
    }

    pub async fn get_text(&self, url: &str) -> Result<String, String> {
        let response = self.send_with_retry(|| self.client.get(url)).await?;
        response.text().await.map_err(|err| err.to_string())
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, String> {
        let response = self.send_with_retry(|| self.client.get(url)).await?;
        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|err| err.to_string())
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        let response = self.send_with_retry(|| self.client.get(url)).await?;
        response.json::<T>().await.map_err(|err| err.to_string())
    }

    pub async fn post_json<B: Serialize>(&self, url: &str, body: &B) -> Result<Response, String> {
        let payload = serde_json::to_value(body).map_err(|err| err.to_string())?;
        self.send_with_retry(|| self.client.post(url).json(&payload))
            .await
    }

    async fn send_with_retry<F>(&self, build_request: F) -> Result<Response, String>
    where
        F: Fn() -> RequestBuilder,
    {
        let mut last_error = "max retries exceeded".to_string();

        for attempt in 1..=self.max_attempts {
            match build_request().send().await {
                Ok(response) if response.status().is_success() => return Ok(response),
                Ok(response) => {
                    let status = response.status();
                    last_error = format!("request failed with status {status}");

                    if attempt == self.max_attempts || !is_retryable_status(status) {
                        return Err(last_error);
                    }
                }
                Err(err) => {
                    last_error = err.to_string();

                    if attempt == self.max_attempts || !is_retryable_error(&err) {
                        return Err(last_error);
                    }
                }
            }

            tokio::time::sleep(retry_delay(attempt)).await;
        }

        Err(last_error)
    }
}

pub async fn get_with_retry(url: &str, max_attempts: u8) -> Result<String, String> {
    RetryingHttpClient::new(Duration::from_secs(10), max_attempts)
        .map_err(|err| err.to_string())?
        .get_text(url)
        .await
}

fn is_retryable_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

fn is_retryable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::REQUEST_TIMEOUT
            | StatusCode::TOO_MANY_REQUESTS
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

fn retry_delay(attempt: u8) -> Duration {
    Duration::from_secs(1 << attempt.min(4))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delay_grows_exponentially_and_caps() {
        assert_eq!(retry_delay(1), Duration::from_secs(2));
        assert_eq!(retry_delay(2), Duration::from_secs(4));
        assert_eq!(retry_delay(3), Duration::from_secs(8));
        assert_eq!(retry_delay(5), Duration::from_secs(16));
    }

    #[test]
    fn retryable_statuses_match_transient_server_failures() {
        assert!(is_retryable_status(reqwest::StatusCode::REQUEST_TIMEOUT));
        assert!(is_retryable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable_status(reqwest::StatusCode::BAD_GATEWAY));
        assert!(!is_retryable_status(reqwest::StatusCode::NOT_FOUND));
        assert!(!is_retryable_status(reqwest::StatusCode::UNAUTHORIZED));
    }
}

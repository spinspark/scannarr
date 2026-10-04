use anyhow::Context;
use reqwest::Client;
use std::time::Duration;
use secrecy::{ExposeSecret, SecretString};
use toml::Value;

pub struct ArrClient {
    http: Client,
}

impl ArrClient {
    pub fn new() -> anyhow::Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .context("Failed to initialize HTTP client")?;

        Ok(Self { http })
    }

    pub async fn test_connection(&self, base_url: &str, api_key: &SecretString) -> anyhow::Result<String> {
        let endpoint = format!("{}/api/v3/system/status", base_url.trim_end_matches('/'));

        let response = self
            .http
            .get(&endpoint)
            .header("X-API-Key", api_key.expose_secret())
            .send()
            .await
            .with_context(|| format!("Failed to connect to {base_url}"))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "HTTP Error {}: API key or URL might be invalid",
                response.status()
            );
        }

        let data: Value = response
            .json()
            .await
            .context("Failed to parse JSON response from server")?;

        let version = data["version"]
            .as_str()
            .unwrap_or("Unknown Version")
            .to_string();

        Ok(version)
    }
}

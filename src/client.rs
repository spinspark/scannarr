use crate::models::{Movie, MovieFile};
use anyhow::Context;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::time::Duration;
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

    pub async fn test_connection(
        &self,
        base_url: &str,
        api_key: &SecretString,
    ) -> anyhow::Result<String> {
        let endpoint = format!("{}/api/v3/system/status", base_url.trim_end_matches('/'));

        let data: Value = self.fetch_json(&endpoint, api_key).await?;

        let version = data["version"]
            .as_str()
            .unwrap_or("Unknown Version")
            .to_string();

        Ok(version)
    }

    pub async fn get_all_movies(
        &self,
        base_url: &str,
        api_key: &SecretString,
    ) -> anyhow::Result<Vec<Movie>> {
        let endpoint = format!("{}/api/v3/movie", base_url.trim_end_matches('/'));

        self.fetch_json(&endpoint, api_key).await
    }

    pub async fn get_movie_file_by_id(
        &self,
        base_url: &str,
        api_key: &SecretString,
        id: u32,
    ) -> anyhow::Result<MovieFile> {
        let endpoint = format!("{}/api/v3/moviefile/{id}", base_url.trim_end_matches('/'));

        self.fetch_json(&endpoint, api_key).await
    }

    async fn fetch_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        api_key: &SecretString,
    ) -> anyhow::Result<T> {
        let response = self
            .http
            .get(endpoint)
            .header("X-API-Key", api_key.expose_secret())
            .send()
            .await
            .with_context(|| format!("Failed to connect to {endpoint}"))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "HTTP Error {}: API key or URL might be invalid",
                response.status()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse JSON response from server")
    }
}

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

    pub async fn get_file_by_movie_id(
        &self,
        base_url: &str,
        api_key: &SecretString,
        movie_id: u32,
    ) -> anyhow::Result<MovieFile> {
        let endpoint = format!("{}/api/v3/moviefile?movieId={movie_id}", base_url.trim_end_matches('/'));

        let mut results: Vec<MovieFile> = self.fetch_json(&endpoint, api_key).await?;
        let movie = results.pop().context("No file found for this movie")?;
        Ok(movie)
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
            let status = response.status();
            let hint = match status.as_u16() {
                401 | 403 => "Check if your API key is missing, expired, or invalid.",
                404 => "The requested resource or endpoint path could not be found.",
                429 => "You are being rate-limited. Please slow down your requests.",
                500..=599 => "The remote server encountered an internal error. Try again later.",
                _ => "Verify your request parameters and network connectivity.",
            };
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unavailable".to_string());

            anyhow::bail!(
                "HTTP Error {status}: {hint}\nURL: {endpoint}\nDetails: {error_body}"
            );
        }

        response
            .json()
            .await
            .context("Failed to parse JSON response from server")
    }
}

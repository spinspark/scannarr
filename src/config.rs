use crate::cli::ApiArgs;
use anyhow::Context;
use secrecy::SecretString;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub url: String,
    pub api_key: SecretString,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub sonarr: Option<ServiceConfig>,
    pub radarr: Option<ServiceConfig>,
}

#[derive(Debug, Deserialize)]
struct RawAppConfig {
    sonarr: Option<RawServiceConfig>,
    radarr: Option<RawServiceConfig>,
}

#[derive(Debug, Deserialize)]
struct RawServiceConfig {
    url: Option<String>,
    api_key: Option<SecretString>,
}

impl AppConfig {
    pub fn load(args: ApiArgs) -> anyhow::Result<Self> {
        let path = args.config;

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file at '{}'", path.display()))?;

        let raw_config: RawAppConfig = toml::from_str(&contents).with_context(|| {
            format!(
                "Invalid TOML structure formatting inside '{}'",
                path.display()
            )
        })?;

        let sonarr_url = args
            .sonarr_url
            .or_else(|| raw_config.sonarr.as_ref().and_then(|f| f.url.clone()))
            .filter(|s| !s.trim().is_empty());

        let sonarr_api_key = args
            .sonarr_api_key
            .or_else(|| raw_config.sonarr.as_ref().and_then(|f| f.api_key.clone()));

        let sonarr = match (sonarr_url, sonarr_api_key) {
            (Some(url), Some(api_key)) => Some(ServiceConfig { url, api_key }),
            _ => None,
        };

        let radarr_url = args
            .radarr_url
            .or_else(|| raw_config.radarr.as_ref().and_then(|f| f.url.clone()))
            .filter(|s| !s.trim().is_empty());

        let radarr_api_key = args
            .radarr_api_key
            .or_else(|| raw_config.radarr.as_ref().and_then(|f| f.api_key.clone()));

        let radarr = match (radarr_url, radarr_api_key) {
            (Some(url), Some(api_key)) => Some(ServiceConfig { url, api_key }),
            _ => None,
        };

        Ok(Self { sonarr, radarr })
    }
}

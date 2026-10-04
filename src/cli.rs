use clap::{Parser, Subcommand};
use std::path::PathBuf;
use secrecy::SecretString;

#[derive(Debug, Parser)]
#[command(
    name = "scannarr",
    version,
    about = "A blazingly fast CLI tool for Sonarr and Radarr, written in Rust."
)]
pub struct Cli {
    /// Custom path to config file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Override Sonarr URL
    #[arg(long, global = true, env = "SONARR_URL")]
    pub sonarr_url: Option<String>,

    /// Override Sonarr API Key
    #[arg(long, global = true, env = "SONARR_API_KEY")]
    pub sonarr_api_key: Option<SecretString>,

    /// Override Radarr URL
    #[arg(long, global = true, env = "RADARR_URL")]
    pub radarr_url: Option<String>,

    /// Override Radarr API Key
    #[arg(long, global = true, env = "RADARR_API_KEY")]
    pub radarr_api_key: Option<SecretString>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Test connectivity to configured services
    Test,
}

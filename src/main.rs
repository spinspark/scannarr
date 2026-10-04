use crate::cli::{Cli, Command};
use crate::client::ArrClient;
use crate::config::AppConfig;
use anyhow::Context;
use clap::Parser;

mod cli;
mod client;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load(&cli).context("Failed to load config")?;

    match &cli.command {
        Command::Test => {
            println!("--- Testing Connections ---");

            let http_client = ArrClient::new()?;

            print!("Sonarr: ");
            if let Some(sonarr) = &config.sonarr {
                match http_client
                    .test_connection(&sonarr.url, &sonarr.api_key)
                    .await
                {
                    Ok(version) => println!("✅ Connected (v{version})"),
                    Err(e) => println!("❌ Failed - {e:#}"),
                }
            } else {
                println!("Not configured (missing URL or API key)");
            }

            print!("Radarr: ");
            if let Some(radarr) = &config.radarr {
                match http_client
                    .test_connection(&radarr.url, &radarr.api_key)
                    .await
                {
                    Ok(version) => println!("✅ Connected (v{version})"),
                    Err(e) => println!("❌ Failed - {e:#}"),
                }
            } else {
                println!("Not configured (missing URL or API key)");
            }
        }
    }
    Ok(())
}

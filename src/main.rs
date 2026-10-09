#![allow(clippy::map_unwrap_or)]

use crate::cli::Cli;
use crate::config::AppConfig;
use anyhow::Context;
use clap::Parser;

mod cli;
mod client;
mod config;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let Cli { api, command } = cli;
    let config = AppConfig::load(api).context("Failed to load config")?;

    command.dispatch(&config).await
}

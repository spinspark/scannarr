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
    let config = AppConfig::load(&cli).context("Failed to load config")?;
    let command = cli.command;

    command.dispatch(&config).await
}

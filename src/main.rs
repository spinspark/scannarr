#![allow(clippy::map_unwrap_or)]

use crate::cli::Cli;
use crate::config::AppConfig;
use anyhow::Context;
use clap::Parser;

mod cache;
mod cli;
mod client;
mod config;
mod handlers;
mod models;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let Cli { api, command } = cli;
    let config = AppConfig::load(api).context("Failed to load config")?;

    with_cancel(command.dispatch(&config)).await
}

/// Wraps an async future, automatically exiting with code 130 if Ctrl+C is pressed
async fn with_cancel<F, T, E>(future: F) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    tokio::select! {
        res = future => res,
        _ = tokio::signal::ctrl_c() => {
            std::process::exit(130);
        }
    }
}

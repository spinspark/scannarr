use crate::config::AppConfig;
use crate::handlers::{handle_search, handle_test};
use clap::{Args, Parser, Subcommand};
use secrecy::SecretString;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("scannarr")
        .join("config.toml")
}

#[derive(Debug, Parser)]
#[command(
    name = "scannarr",
    version,
    about = "A blazingly fast CLI tool for Sonarr and Radarr, written in Rust."
)]
pub struct Cli {
    #[command(flatten)]
    pub api: ApiArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct ApiArgs {
    /// Custom path to config file
    #[arg(short, long, global = true, default_value_os_t = default_config_path())]
    pub config: PathBuf,

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
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Test connectivity to configured services
    Test,
    Search {
        #[command(flatten)]
        filters: Filters,

        #[arg(long, default_value_t = SortBy::default() )]
        sort: SortBy,
    },
}

#[derive(Debug, Args)]
pub struct Filters {
    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub year: Option<u16>,

    #[arg(long)]
    pub monitored: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SortField {
    #[default]
    Title,
    Year,
    Monitored,
}

#[derive(Debug)]
pub struct ParseSortFieldError;

impl fmt::Display for ParseSortFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "provided string was not a valid sort field")
    }
}

impl Error for ParseSortFieldError {}

impl FromStr for SortField {
    type Err = ParseSortFieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "title" => Ok(Self::Title),
            "year" => Ok(Self::Year),
            "monitored" => Ok(Self::Monitored),
            _ => Err(ParseSortFieldError),
        }
    }
}

impl fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Title => "title",
            Self::Year => "year",
            Self::Monitored => "monitored",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug)]
pub struct ParseSortDirectionError;

impl fmt::Display for ParseSortDirectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "provided string was not a valid sort direction")
    }
}

impl Error for ParseSortDirectionError {}

impl FromStr for SortDirection {
    type Err = ParseSortDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Ok(Self::Ascending),
            "desc" | "descending" => Ok(Self::Descending),
            _ => Err(ParseSortDirectionError),
        }
    }
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Default)]
pub struct SortBy {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Debug)]
pub enum ParseSortByError {
    Field(ParseSortFieldError),
    Direction(ParseSortDirectionError),
}

impl From<ParseSortFieldError> for ParseSortByError {
    fn from(err: ParseSortFieldError) -> Self {
        Self::Field(err)
    }
}

impl From<ParseSortDirectionError> for ParseSortByError {
    fn from(err: ParseSortDirectionError) -> Self {
        Self::Direction(err)
    }
}

impl fmt::Display for ParseSortByError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field(err) => write!(f, "{err}"),
            Self::Direction(err) => write!(f, "{err}"),
        }
    }
}

impl Error for ParseSortByError {}

impl FromStr for SortBy {
    type Err = ParseSortByError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        let field = SortField::from_str(parts[0])?;

        let direction = match parts.get(1) {
            Some(direction) => SortDirection::from_str(direction)?,
            None => SortDirection::default(),
        };

        Ok(Self { field, direction })
    }
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { field, direction } = self;
        write!(f, "{field}:{direction}")
    }
}

impl Command {
    pub async fn dispatch(self, config: &AppConfig) -> anyhow::Result<()> {
        match self {
            Self::Test => handle_test(config).await,
            Self::Search { filters, sort } => handle_search(config, filters, sort).await,
        }
    }
}

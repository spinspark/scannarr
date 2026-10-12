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

#[derive(Debug, Clone, Copy)]
pub enum MovieField {
    Title,
    Year,
    Monitored,
    CustomFormatScore,
}

impl MovieField {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Year => "year",
            Self::Monitored => "monitored",
            Self::CustomFormatScore => "custom-format-score",
        }
    }

    pub const fn as_header(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Year => "Year",
            Self::Monitored => "Monitored",
            Self::CustomFormatScore => "Custom Format Score",
        }
    }
}

#[derive(Debug)]
pub struct ParseMovieFieldError;

impl fmt::Display for ParseMovieFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "provided string was not a valid movie field")
    }
}

impl Error for ParseMovieFieldError {}

impl FromStr for MovieField {
    type Err = ParseMovieFieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "title" => Ok(Self::Title),
            "year" => Ok(Self::Year),
            "monitored" => Ok(Self::Monitored),
            "custom-format-score" => Ok(Self::CustomFormatScore),
            _ => Err(ParseMovieFieldError),
        }
    }
}

impl fmt::Display for MovieField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
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

        #[arg(long, default_value = "title")]
        sort: SortOrder,

        #[arg(long, value_delimiter = ',', default_value = "title,year,monitored")]
        fields: Vec<MovieField>,

        #[arg(long)]
        refresh: bool,
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

    #[arg(long)]
    pub custom_format_score: Option<i32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
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
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct SortOrder {
    pub field: MovieField,
    pub direction: SortDirection,
}

#[derive(Debug)]
pub enum ParseSortOrderError {
    Field(ParseMovieFieldError),
    Direction(ParseSortDirectionError),
}

impl From<ParseMovieFieldError> for ParseSortOrderError {
    fn from(err: ParseMovieFieldError) -> Self {
        Self::Field(err)
    }
}

impl From<ParseSortDirectionError> for ParseSortOrderError {
    fn from(err: ParseSortDirectionError) -> Self {
        Self::Direction(err)
    }
}

impl fmt::Display for ParseSortOrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field(err) => write!(f, "{err}"),
            Self::Direction(err) => write!(f, "{err}"),
        }
    }
}

impl Error for ParseSortOrderError {}

impl FromStr for SortOrder {
    type Err = ParseSortOrderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        let field = MovieField::from_str(parts[0])?;

        let direction = match parts.get(1) {
            Some(s) => SortDirection::from_str(s)?,
            None => SortDirection::default(),
        };

        Ok(Self { field, direction })
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { field, direction } = self;
        write!(f, "{field}:{direction}")
    }
}

impl Command {
    pub async fn dispatch(self, config: &AppConfig) -> anyhow::Result<()> {
        match self {
            Self::Test => handle_test(config).await,
            Self::Search {
                filters,
                sort,
                fields,
                refresh,
            } => handle_search(config, filters, sort, &fields, refresh).await,
        }
    }
}

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimalResource {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub id: u32,
    pub imdb_id: String,
    pub tmdb_id: u32,
    pub title: String,
    pub clean_title: String,
    pub year: u16,
    pub original_language: MinimalResource,
    pub status: String,
    pub runtime: u16,
    pub studio: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certification: Option<String>,
    pub genres: Vec<String>,
    pub monitored: bool,
    pub has_file: bool,
    pub is_available: bool,
    pub added: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_cinemas: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digital_release: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<Timestamp>,
    #[serde(rename = "movieFile", skip_serializing_if = "Option::is_none")]
    pub file: Option<MovieFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieFile {
    pub id: u32,
    pub movie_id: u32,
    pub size: u64,
    pub relative_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    pub languages: Vec<MinimalResource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_format_score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_formats: Option<Vec<MinimalResource>>,
    pub date_added: Timestamp,
    pub quality: MediaQuality,
    pub media_info: MediaInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaQuality {
    pub quality: Quality,
    pub revision: Revision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quality {
    pub id: u32,
    pub name: String,
    pub source: String,
    pub resolution: u16,
    pub modifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    pub version: u32,
    pub real: u32,
    pub is_repack: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub audio_bitrate: u32,
    pub audio_channels: f64,
    pub audio_codec: String,
    pub audio_languages: String,
    pub video_bit_depth: u8,
    pub video_bitrate: u32,
    pub video_codec: String,
    pub video_fps: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_dynamic_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_dynamic_range_type: Option<String>,
    pub subtitles: String,
}

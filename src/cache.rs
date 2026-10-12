use jiff::Timestamp;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub updated_at: Timestamp,
    pub value: T,
}

fn default_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("scannarr")
        .join("movies_cache.json")
}

pub fn write_to_cache<T: Serialize>(value: T) -> anyhow::Result<()> {
    let path = default_cache_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let entry = CacheEntry {
        updated_at: Timestamp::now(),
        value,
    };

    let json_string = serde_json::to_string(&entry)?;

    fs::write(path, json_string)?;

    Ok(())
}

pub fn read_from_cache<T: DeserializeOwned>() -> Option<CacheEntry<T>> {
    let path = default_cache_path();
    if !path.exists() {
        return None;
    }

    let raw_content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw_content).ok()
}

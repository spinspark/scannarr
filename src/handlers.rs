use crate::cache;
use crate::cli::{Filters, MovieField, SortDirection, SortOrder};
use crate::client::ArrClient;
use crate::config::AppConfig;
use crate::models::Movie;
use comfy_table::Table;
use jiff::Timestamp;

pub async fn handle_test(config: &AppConfig) -> anyhow::Result<()> {
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
    Ok(())
}

pub async fn handle_search(
    config: &AppConfig,
    filters: Filters,
    sort_by: SortOrder,
    fields: &[MovieField],
    force_refresh: bool,
) -> anyhow::Result<()> {
    println!("Fetching movies...");

    let mut source_movies = Vec::new();
    let mut cache_loaded = false;

    if !force_refresh && let Some(entry) = cache::read_from_cache::<Vec<Movie>>() {
        let cache_age = Timestamp::now().duration_since(entry.updated_at);

        if cache_age.as_hours() < 24 {
            source_movies = entry.value;
            cache_loaded = true;
        }
    }

    if !cache_loaded {
        if let Some(radarr) = &config.radarr {
            let http_client = ArrClient::new()?;
            source_movies = http_client
                .get_all_movies(&radarr.url, &radarr.api_key)
                .await?;

            for movie in &mut source_movies {
                if !movie.has_file {
                    continue;
                }
                let file = http_client
                    .get_file_by_movie_id(&radarr.url, &radarr.api_key, movie.id)
                    .await?;
                movie.file = Some(file);
            }

            cache::write_to_cache(source_movies.clone())?;
        } else {
            println!("Radarr: Not configured (missing URL or API key)");
            return Ok(());
        }
    }

    let mut selected_movies: Vec<&Movie> = source_movies.iter().collect();

    if let Some(title) = filters.title {
        selected_movies.retain(|m| m.title == title || m.clean_title == title);
    }

    if let Some(year) = filters.year {
        selected_movies.retain(|m| m.year == year);
    }

    if filters.monitored {
        selected_movies.retain(|m| m.monitored);
    }

    if let Some(score) = filters.custom_format_score {
        selected_movies.retain(|m| {
            m.file
                .as_ref()
                .and_then(|f| f.custom_format_score)
                .is_some_and(|movie_score| movie_score == score)
        });
    }

    selected_movies.sort_unstable_by(|a, b| {
        let ordering = match sort_by.field {
            MovieField::Title => a.title.cmp(&b.title),
            MovieField::Year => a.year.cmp(&b.year),
            MovieField::Monitored => a.monitored.cmp(&b.monitored),
            MovieField::CustomFormatScore => {
                let score_a = a.file.as_ref().and_then(|f| f.custom_format_score);
                let score_b = b.file.as_ref().and_then(|f| f.custom_format_score);
                score_a.cmp(&score_b)
            }
        };

        if sort_by.direction == SortDirection::Descending {
            ordering.reverse()
        } else {
            ordering
        }
    });

    let mut table = Table::new();
    let header_row: Vec<&str> = fields.iter().map(|f| f.as_header()).collect();
    table.set_header(header_row);

    for movie in selected_movies {
        let row: Vec<String> = fields
            .iter()
            .map(|field| match field {
                MovieField::Title => movie.title.clone(),
                MovieField::Year => movie.year.to_string(),
                MovieField::Monitored => movie.monitored.to_string(),
                MovieField::CustomFormatScore => movie
                    .file
                    .as_ref()
                    .and_then(|f| f.custom_format_score)
                    .map(|score| score.to_string())
                    .unwrap_or_default(),
            })
            .collect();
        table.add_row(row);
    }
    println!("{table}");

    Ok(())
}

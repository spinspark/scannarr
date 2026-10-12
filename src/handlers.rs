use crate::cache;
use crate::cli::{Filters, SortDirection, SortField, SortOrder};
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

    selected_movies.sort_unstable_by(|a, b| {
        let ordering = match sort_by.field {
            SortField::Title => a.title.cmp(&b.title),
            SortField::Year => a.year.cmp(&b.year),
            SortField::Monitored => a.monitored.cmp(&b.monitored),
        };

        if sort_by.direction == SortDirection::Descending {
            ordering.reverse()
        } else {
            ordering
        }
    });

    let mut table = Table::new();
    table.set_header(["Title", "Year", "Monitored"]);

    for movie in selected_movies {
        let year_string = movie.year.to_string();
        let monitored_string = movie.monitored.to_string();

        table.add_row([&movie.title, &year_string, &monitored_string]);
    }
    println!("{table}");

    Ok(())
}

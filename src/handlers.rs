use crate::cli::{Filters, SortBy, SortDirection, SortField};
use crate::client::ArrClient;
use crate::config::AppConfig;
use comfy_table::Table;

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
    sort_by: SortBy,
) -> anyhow::Result<()> {
    println!("Fetching movies...");

    let http_client = ArrClient::new()?;

    if let Some(radarr) = &config.radarr {
        let mut movies = http_client
            .get_all_movies(&radarr.url, &radarr.api_key)
            .await?;

        if let Some(title) = filters.title {
            movies.retain(|m| m.title == title || m.clean_title == title);
        }

        if let Some(year) = filters.year {
            movies.retain(|m| m.year == year);
        }

        if filters.monitored {
            movies.retain(|m| m.monitored);
        }

        movies.sort_by(|a, b| {
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

        for movie in movies {
            table.add_row([
                movie.title,
                movie.year.to_string(),
                movie.monitored.to_string(),
            ]);
        }
        println!("{table}");
    } else {
        println!("Radarr: Not configured (missing URL or API key)");
        return Ok(());
    }

    Ok(())
}

use anyhow::Context;
use nlhousefinder::scraping::{pararius::ParariusScraper, WebsiteScraper};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let scraper = ParariusScraper::default();
    let properties = scraper.list_properties().await?;
    let first_property = properties.first().context("no properties")?;
    let full_property = scraper.scrape_property(first_property.clone()).await?;

    println!("{full_property:#?}");

    Ok(())
}

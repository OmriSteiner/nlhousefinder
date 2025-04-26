use anyhow::Context;
use clap::{Parser, ValueEnum};
use nlhousefinder::scraping::{
    huurwoningen::HuurwoningenScraper, ikwilhuren::IkwilhurenScraper, pararius::ParariusScraper,
    rotterdamwonen::RotterdamWonenScraper, WebsiteScraper,
};

#[derive(Parser)]
struct Args {
    website: Website,
}

#[derive(ValueEnum, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Website {
    Pararius,
    Huurwoningen,
    Ikwilhuren,
    RotterdamWonen,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let scraper: Box<dyn WebsiteScraper> = match args.website {
        Website::Pararius => Box::new(ParariusScraper::default()),
        Website::Huurwoningen => Box::new(HuurwoningenScraper::default()),
        Website::Ikwilhuren => Box::new(IkwilhurenScraper::default()),
        Website::RotterdamWonen => Box::new(RotterdamWonenScraper::default()),
    };

    let properties = scraper.list_properties().await?;
    let first_property = properties.first().context("no properties")?;
    let full_property = scraper.full(first_property.clone()).await?;

    println!("{full_property:#?}");

    Ok(())
}

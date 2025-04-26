use anyhow::Context;
use futures::future::BoxFuture;
use itertools::Itertools;
use reqwest;
use scraper::{ElementRef, Html, Selector};

use super::{
    utils::SelectExt, FullScrapeResult, PartialScrapeResult, ScrapeResult, WebsiteScraper,
};

pub struct IkwilhurenScraper {
    listing_selector: Selector,
    title_selector: Selector,
    price_selector: Selector,
    area_selector: Selector,
    map_selector: Selector,
}

impl Default for IkwilhurenScraper {
    fn default() -> Self {
        Self {
            listing_selector: Selector::parse(".card-woning").unwrap(),
            title_selector: Selector::parse(".card-title a").unwrap(),
            price_selector: Selector::parse(".fw-bold").unwrap(),
            area_selector: Selector::parse("span:nth-child(2)").unwrap(),
            map_selector: Selector::parse("#maplibre-object").unwrap(),
        }
    }
}

impl WebsiteScraper for IkwilhurenScraper {
    fn list_properties(&self) -> BoxFuture<anyhow::Result<Vec<ScrapeResult>>> {
        Box::pin(async {
            let response = reqwest::get("https://ikwilhuren.nu/aanbod/?sort=aanbodDESC")
                .await?
                .error_for_status()?
                .text()
                .await?;

            let document = Html::parse_document(&response);
            let listings = document.select(&self.listing_selector);

            let results = listings
                .map(|listing| {
                    let title_element = listing.select_one(&self.title_selector)?;
                    let url = format!(
                        "https://ikwilhuren.nu{}",
                        title_element.attr("href").context("missing URL")?
                    );

                    let title = title_element
                        .text()
                        .next()
                        .context("no title")?
                        .trim()
                        .to_string();

                    let price_element = listing.select_one(&self.price_selector)?;
                    let raw_price = price_element.text().next().context("no price")?;
                    let price = raw_price
                        .split(" ")
                        .nth(1)
                        .unwrap()
                        .replace(".", "")
                        .replace(",", "")
                        .replace("-", "")
                        .parse()
                        .with_context(|| format!("Invalid price format: {raw_price}"))?;

                    let price_parent =
                        ElementRef::wrap(price_element.parent().context("no parent of price")?)
                            .unwrap();

                    let area = price_parent.select_one_text(&self.area_selector)?;
                    let area = area
                        .split(" ")
                        .next()
                        .unwrap_or(area)
                        .parse()
                        .with_context(|| format!("invalid area: {area}"))?;

                    anyhow::Ok(ScrapeResult::Partial(PartialScrapeResult {
                        title,
                        price,
                        url,
                        area,
                    }))
                })
                .try_collect()?;

            Ok(results)
        })
    }

    fn scrape_property(
        &self,
        partial: PartialScrapeResult,
    ) -> BoxFuture<anyhow::Result<FullScrapeResult>> {
        Box::pin(async move {
            let response = reqwest::get(&partial.url)
                .await?
                .error_for_status()?
                .text()
                .await?;

            let document = Html::parse_document(&response);
            let map = document.select_one(&self.map_selector)?;

            let longitude = map.attr("data-lng").context("no longitude")?;
            let longitude: f64 = longitude
                .parse()
                .with_context(|| format!("invalid longitude: {longitude}"))?;

            let latitude = map.attr("data-lat").context("no latitude")?;
            let latitude: f64 = latitude
                .parse()
                .with_context(|| format!("invalid latitude: {latitude}"))?;

            Ok(FullScrapeResult {
                partial,
                location: geo::Point::new(longitude, latitude),
            })
        })
    }
}

use anyhow::Context;
use futures::future::BoxFuture;
use itertools::Itertools;
use scraper::{Html, Selector};

use super::{
    utils::SelectExt, FullScrapeResult, PartialScrapeResult, ScrapeResult, WebsiteScraper,
};

pub struct RotterdamWonenScraper {
    houses_selector: Selector,
    area_selector: Selector,
}

impl Default for RotterdamWonenScraper {
    fn default() -> Self {
        Self {
            houses_selector: Selector::parse("div.property-list-item").unwrap(),
            // I'm not 100% sure this will be consistent across all listings, but
            // it seems to be the case for all listings currently listed.
            // Maybe if there are more "meta" items the order will change.
            area_selector: Selector::parse(
                ".property-meta-item.first-item > .property-meta-number",
            )
            .unwrap(),
        }
    }
}

impl WebsiteScraper for RotterdamWonenScraper {
    fn list_properties(&self) -> BoxFuture<anyhow::Result<Vec<ScrapeResult>>> {
        Box::pin(async {
            let response = reqwest::get("https://www.rotterdamwonen.nl/aanbod/?sortby=date-desc")
                .await?
                .error_for_status()?
                .text()
                .await?;

            let document = Html::parse_document(&response);
            let houses = document.select(&self.houses_selector);

            Ok(houses
                .into_iter()
                .map(|house| {
                    let title = house.attr("data-title").unwrap_or_default().to_string();
                    let price_raw = house.attr("data-price").context("no price")?;
                    let price = price_raw
                        .split_whitespace()
                        .last()
                        .with_context(|| format!("invalid price: {price_raw}"))?
                        .parse()
                        .with_context(|| format!("invalid price: {price_raw}"))?;

                    let area_raw = house
                        .select_one_text(&self.area_selector)
                        .context("no area")?;
                    let area = area_raw
                        .parse()
                        .with_context(|| format!("invalid area: {area_raw}"))?;

                    let url = house.attr("data-link").context("no URL")?.to_string();

                    let latitude_raw = house.attr("data-lat").context("no latitude")?;
                    let longitude_raw = house.attr("data-long").context("no longitude")?;

                    let latitude = latitude_raw
                        .parse()
                        .with_context(|| format!("invalid latitude: {latitude_raw}"))?;
                    let longitude = longitude_raw
                        .parse()
                        .with_context(|| format!("invalid longitude: {longitude_raw}"))?;

                    anyhow::Ok(ScrapeResult::Full(FullScrapeResult {
                        partial: PartialScrapeResult {
                            title,
                            price,
                            url,
                            area,
                        },
                        location: geo::Point::new(longitude, latitude),
                    }))
                })
                .try_collect()?)
        })
    }

    fn scrape_property(
        &self,
        _partial: PartialScrapeResult,
    ) -> BoxFuture<anyhow::Result<FullScrapeResult>> {
        Box::pin(futures::future::ready(Err(anyhow::anyhow!(
            "RotterdamWonen does not support scraping individual properties"
        ))))
    }
}

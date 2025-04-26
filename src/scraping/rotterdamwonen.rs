use anyhow::Context;
use futures::future::BoxFuture;
use itertools::Itertools;
use scraper::{Html, Selector};

use super::{
    utils::SelectExt, FullScrapeResult, PartialScrapeResult, ScrapeResult, WebsiteScraper,
};

pub struct RotterdamWonenScraper {
    houses_selector: Selector,
    //title_selector: Selector,
    //price_selector: Selector,
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

                    anyhow::Ok(ScrapeResult::Partial(PartialScrapeResult {
                        title,
                        price,
                        url,
                        area,
                    }))
                })
                .try_collect()?)
        })
    }

    fn scrape_property(
        &self,
        partial: PartialScrapeResult,
    ) -> BoxFuture<anyhow::Result<FullScrapeResult>> {
        Box::pin(async {
            Ok(FullScrapeResult {
                partial,
                location: geo::Point::new(0.0, 0.0), // TODO: Implement this
            })
        })
    }
}

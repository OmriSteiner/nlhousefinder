use anyhow::Context;
use futures::future::BoxFuture;
use itertools::Itertools;
use scraper::{Html, Selector};

use super::{utils::SelectExt, FullScrapeResult, PartialScrapeResult, WebsiteScraper};

pub struct HuurwoningenScraper {
    houses_selector: Selector,
    price_selector: Selector,
    title_selector: Selector,
    area_selector: Selector,
    map_selector: Selector,
}

impl Default for HuurwoningenScraper {
    fn default() -> Self {
        Self {
            houses_selector: Selector::parse("li.search-list__item--listing").unwrap(),
            price_selector: Selector::parse("div.listing-search-item__price").unwrap(),
            title_selector: Selector::parse("a.listing-search-item__link--title").unwrap(),
            area_selector: Selector::parse("li.illustrated-features__item--surface-area").unwrap(),
            map_selector: Selector::parse("wc-detail-map").unwrap(),
        }
    }
}

impl WebsiteScraper for HuurwoningenScraper {
    fn list_properties(&self) -> BoxFuture<anyhow::Result<Vec<super::PartialScrapeResult>>> {
        Box::pin(async {
            let response = reqwest::get(
                "https://www.huurwoningen.nl/in/rotterdam/?sort=published_at&direction=desc",
            )
            .await?
            .error_for_status()?
            .text()
            .await?;

            let document = Html::parse_document(&response);
            let houses = document.select(&self.houses_selector);

            Ok(houses
                .into_iter()
                .map(|house| {
                    let price_raw = house
                        .select_one_text(&self.price_selector)
                        .context("no price")?;

                    let price: usize = price_raw
                        // It starts with a euro sign and an NBSP,
                        // so we use split_whitespace to handle that.
                        .split_whitespace()
                        .nth(1)
                        .with_context(|| format!("invalid price {price_raw}"))?
                        .replace(".", "")
                        .parse()
                        .with_context(|| format!("invalid price {price_raw}"))?;

                    let title = house.select_one(&self.title_selector).context("no title")?;
                    let uri = title.attr("href").context("no href in title")?;
                    let url = format!("https://www.huurwoningen.nl{uri}");

                    let area_raw = house
                        .select_one_text(&self.area_selector)
                        .context("no area")?;
                    let area = area_raw
                        .strip_suffix(" m²")
                        .unwrap_or(area_raw)
                        .parse()
                        .with_context(|| format!("invalid area {area_raw}"))?;

                    anyhow::Ok(PartialScrapeResult {
                        title: "".to_string(),
                        price,
                        url,
                        area,
                    })
                })
                .try_collect()?)
        })
    }

    fn scrape_property(
        &self,
        partial: PartialScrapeResult,
    ) -> BoxFuture<anyhow::Result<FullScrapeResult>> {
        Box::pin(async {
            let response = reqwest::get(&partial.url)
                .await
                .and_then(reqwest::Response::error_for_status)
                .with_context(|| format!("failed to GET property at {}", partial.url))?
                .text()
                .await?;

            let appartment_document = Html::parse_document(&response);
            let map = appartment_document.select_one(&self.map_selector)?;
            let longitude: f64 = map
                .attr("data-longitude")
                .context("no longitude")?
                .parse()
                .context("invalid longitude")?;
            let latitude: f64 = map
                .attr("data-latitude")
                .context("no latitude")?
                .parse()
                .context("invalid latitude")?;

            Ok(FullScrapeResult {
                partial,
                location: geo::Point::new(longitude, latitude),
            })
        })
    }
}

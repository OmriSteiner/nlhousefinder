use anyhow::Context;
use futures::future::BoxFuture;
use itertools::Itertools;
use scraper::{Html, Selector};

use super::{utils::SelectExt, FullScrapeResult, PartialScrapeResult, WebsiteScraper};

pub struct ParariusScraper {
    // Unfortunately `scraper` doesn't have a compile-time checked way to define selectors.
    houses_selector: Selector,
    title_selector: Selector,
    #[allow(unused)]
    subtitle_selector: Selector,
    map_selector: Selector,
    price_selector: Selector,
    area_selector: Selector,
}

impl Default for ParariusScraper {
    fn default() -> Self {
        Self {
            houses_selector: Selector::parse("section.listing-search-item--for-rent").unwrap(),
            title_selector: Selector::parse("a.listing-search-item__link--title").unwrap(),
            // subtitle class has a ' at the end, but it's an invalid CSS class name, so we use the ^=
            // operator to match the start of the class name
            subtitle_selector: Selector::parse("div[class^=listing-search-item__sub-title]")
                .unwrap(),
            map_selector: Selector::parse("wc-detail-map").unwrap(),
            price_selector: Selector::parse("div.listing-search-item__price").unwrap(),
            area_selector: Selector::parse(".illustrated-features__item--surface-area").unwrap(),
        }
    }
}

impl WebsiteScraper for ParariusScraper {
    fn list_properties(&self) -> BoxFuture<anyhow::Result<Vec<PartialScrapeResult>>> {
        Box::pin(async {
            // By default querying this URL returns results sorted by newest first
            let response = reqwest::get("https://www.pararius.com/apartments/rotterdam")
                .await?
                .error_for_status()?
                .text()
                .await?;

            let document = Html::parse_document(&response);
            let houses = document.select(&self.houses_selector);

            Ok(houses
                .into_iter()
                .map(|house| {
                    let title = house.select_one(&self.title_selector)?;
                    let raw_address = title.text().next().context("no address")?.trim();
                    let address = raw_address
                        .split_once(" ")
                        .map(|(_, rest)| rest)
                        .unwrap_or(raw_address);

                    //let zipcode = house.select_one_text(&self.subtitle_selector)?
                    //    // "1234 AB" -> 7
                    //    .split_at_checked(7)
                    //    .context("invalid zipcode")?
                    //    .0;

                    let uri = title.attr("href").context("no link")?;
                    let url = format!("https://pararius.com{}", uri);

                    let raw_price = house.select_one_text(&self.price_selector)?;
                    let price: usize = if raw_price == "Price on request" {
                        9999
                    } else {
                        raw_price
                            .split(" ")
                            .next()
                            .unwrap()
                            .replace("€", "")
                            .replace(",", "")
                            .parse()
                            .with_context(|| format!("invalid price: {raw_price}"))?
                    };

                    let area = house.select_one_text(&self.area_selector)?;
                    let area = area
                        .split(" ")
                        .next()
                        .unwrap_or(area)
                        .parse()
                        .with_context(|| format!("invalid area: {area}"))?;

                    anyhow::Ok(PartialScrapeResult {
                        title: address.to_string(),
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

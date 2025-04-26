use futures::future::BoxFuture;
use serde::Deserialize;

use super::{FullScrapeResult, PartialScrapeResult, ScrapeResult, WebsiteScraper};

#[derive(Default)]
pub struct VerraMakelaarsScraper;

impl WebsiteScraper for VerraMakelaarsScraper {
    fn list_properties(&self) -> BoxFuture<anyhow::Result<Vec<ScrapeResult>>> {
        Box::pin(async {
            let houses: Vec<Listing> =
                reqwest::get("https://www.verra.nl/nl/realtime-listings/consumer")
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;

            Ok(houses
                .into_iter()
                .filter(|house| house.is_rentals && house.status == "Beschikbaar")
                .map(|house| {
                    ScrapeResult::Full(FullScrapeResult {
                        partial: PartialScrapeResult {
                            title: house.address,
                            price: house.price,
                            url: format!("https://www.verra.nl{}/", house.url),
                            area: house.area,
                        },
                        location: geo::Point::new(house.longitude, house.latitude),
                    })
                })
                .collect())
        })
    }

    fn scrape_property(
        &self,
        _partial: PartialScrapeResult,
    ) -> BoxFuture<anyhow::Result<FullScrapeResult>> {
        Box::pin(futures::future::ready(Err(anyhow::anyhow!(
            "VerraMakelaars does not support scraping individual properties"
        ))))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Listing {
    address: String,
    #[serde(rename = "rentalsPrice")]
    price: usize,
    #[serde(rename = "livingSurface")]
    area: u32,
    is_rentals: bool,
    status: String,
    url: String,

    #[serde(rename = "lng")]
    longitude: f64,
    #[serde(rename = "lat")]
    latitude: f64,
}

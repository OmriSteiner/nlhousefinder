pub mod pararius;
mod utils;

use futures::future::BoxFuture;

pub trait WebsiteScraper {
    /// List the most recent properties on the website.
    /// Return their links.
    fn list_properties(&self) -> BoxFuture<anyhow::Result<Vec<PartialScrapeResult>>>;

    /// Scrape a given property
    fn scrape_property(
        &self,
        partial: PartialScrapeResult,
    ) -> BoxFuture<anyhow::Result<FullScrapeResult>>;
}

#[derive(Debug, Clone)]
pub struct PartialScrapeResult {
    #[allow(unused)]
    title: String,
    pub(super) price: usize,
    pub(super) url: String,
    pub(super) area: u32,
}

#[derive(Debug)]
pub struct FullScrapeResult {
    #[allow(unused)]
    partial: PartialScrapeResult,
    pub(super) location: geo::Point<f64>,
}

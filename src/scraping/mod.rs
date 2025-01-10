pub mod pararius;
mod utils;

#[allow(async_fn_in_trait)]
pub trait WebsiteScraper {
    /// List the most recent properties on the website.
    /// Return their links.
    async fn list_properties(&self) -> anyhow::Result<Vec<PartialScrapeResult>>;

    /// Scrape a given property
    async fn scrape_property(
        &self,
        partial: PartialScrapeResult,
    ) -> anyhow::Result<FullScrapeResult>;
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

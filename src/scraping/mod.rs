pub mod pararius;

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
    title: String,
    pub(super) price: usize,
    pub(super) url: String,
}

#[derive(Debug)]
pub struct FullScrapeResult {
    partial: PartialScrapeResult,
    pub(super) location: geo::Point<f64>,
}

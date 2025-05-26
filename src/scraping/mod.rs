pub mod huurwoningen;
pub mod ikwilhuren;
pub mod pararius;
pub mod rotterdamwonen;
pub mod verra;
pub mod vesteda;
mod utils;

use std::ops::Deref;

use futures::future::BoxFuture;

pub trait WebsiteScraper {
    /// List the most recent properties on the website.
    /// Return their links.
    fn list_properties(&self) -> BoxFuture<anyhow::Result<Vec<ScrapeResult>>>;

    /// Scrape a given property
    fn scrape_property(
        &self,
        partial: PartialScrapeResult,
    ) -> BoxFuture<anyhow::Result<FullScrapeResult>>;

    fn full(&self, result: ScrapeResult) -> BoxFuture<anyhow::Result<FullScrapeResult>> {
        match result {
            ScrapeResult::Partial(partial) => self.scrape_property(partial),
            ScrapeResult::Full(full) => Box::pin(futures::future::ready(Ok(full))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartialScrapeResult {
    #[allow(unused)]
    title: String,
    pub(super) price: usize,
    pub(super) url: String,
    pub(super) area: u32,
}

#[derive(Debug, Clone)]
pub struct FullScrapeResult {
    partial: PartialScrapeResult,
    pub(super) location: geo::Point<f64>,
}

#[derive(Clone)]
pub enum ScrapeResult {
    Partial(PartialScrapeResult),
    Full(FullScrapeResult),
}

impl Deref for ScrapeResult {
    type Target = PartialScrapeResult;

    fn deref(&self) -> &Self::Target {
        match self {
            ScrapeResult::Partial(partial) => partial,
            ScrapeResult::Full(full) => &full.partial,
        }
    }
}

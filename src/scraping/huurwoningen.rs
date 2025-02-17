use super::WebsiteScraper;

#[derive(Default)]
pub struct HuurwoningenScraper;

impl WebsiteScraper for HuurwoningenScraper {
    fn list_properties(
        &self,
    ) -> futures::future::BoxFuture<anyhow::Result<Vec<super::PartialScrapeResult>>> {
        todo!()
    }

    fn scrape_property(
        &self,
        _partial: super::PartialScrapeResult,
    ) -> futures::future::BoxFuture<anyhow::Result<super::FullScrapeResult>> {
        todo!()
    }
}

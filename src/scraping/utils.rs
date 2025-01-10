use anyhow::Context;
use scraper::{ElementRef, Html, Selector};

pub(super) trait SelectExt<'a> {
    fn select_one(&'a self, selector: &Selector) -> anyhow::Result<ElementRef<'a>>;
    fn select_one_text(&'a self, selector: &Selector) -> anyhow::Result<&'a str> {
        Ok(self
            .select_one(selector)?
            .text()
            .next()
            .context("no text")?
            .trim())
    }
}

impl<'a> SelectExt<'a> for Html {
    fn select_one(&'a self, selector: &Selector) -> anyhow::Result<ElementRef<'a>> {
        self.select(selector)
            .next()
            .with_context(|| format!("no element matching {selector:?}"))
    }
}

impl<'a> SelectExt<'a> for ElementRef<'a> {
    fn select_one(&'a self, selector: &Selector) -> anyhow::Result<ElementRef<'a>> {
        self.select(selector)
            .next()
            .with_context(|| format!("no element matching {selector:?}"))
    }
}

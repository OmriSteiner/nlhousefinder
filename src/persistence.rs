use std::collections::HashSet;

use anyhow::Context;
use sqlx::sqlite::SqlitePool;

pub(super) struct Persistence {
    pool: SqlitePool,
}

impl Persistence {
    pub(super) async fn new() -> anyhow::Result<Self> {
        let pool = SqlitePool::connect("sqlite://database.db")
            .await
            .context("failed to open db")?;
        sqlx::migrate!()
            .run(&pool)
            .await
            .context("migrations failed")?;
        Ok(Self { pool })
    }

    pub(super) async fn save_property(&self, url: &str) -> anyhow::Result<()> {
        sqlx::query!("INSERT INTO properties (url) VALUES (?)", url)
            .execute(&self.pool)
            .await
            .context("failed to save property")?;
        Ok(())
    }

    pub(super) async fn list_properties(&self) -> anyhow::Result<HashSet<String>> {
        let result = sqlx::query!("SELECT url FROM properties")
            .fetch_all(&self.pool)
            .await
            .context("failed to list properties")?;
        Ok(result.iter().map(|row| row.url.clone()).collect())
    }
}

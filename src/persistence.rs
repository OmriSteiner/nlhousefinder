use std::collections::HashSet;

use anyhow::Context;
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
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

    pub(super) async fn add_subscriber(&self, chat_id: i64) -> anyhow::Result<()> {
        sqlx::query!("INSERT INTO subscribers (chat_id) VALUES (?)", chat_id)
            .execute(&self.pool)
            .await
            .context("failed to add subscriber")?;
        Ok(())
    }

    pub(super) async fn list_subscribers(&self) -> anyhow::Result<Vec<i64>> {
        let result = sqlx::query!("SELECT chat_id FROM subscribers")
            .fetch_all(&self.pool)
            .await
            .context("failed to list subscribers")?;
        Ok(result.iter().map(|row| row.chat_id).collect())
    }
}

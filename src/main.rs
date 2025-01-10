use nlhousefinder::run_bot;
use teloxide::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let bot = Bot::from_env();
    run_bot(bot).await
}

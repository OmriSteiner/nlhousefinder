mod location;
mod persistence;
pub mod scraping;

use std::sync::Arc;

use geo::Contains;
use location::DESIRED_LOCATION;
use persistence::Persistence;
use scraping::{pararius::ParariusScraper, WebsiteScraper};
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    prelude::*,
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Intro message")]
    Start,
    #[command(description = "Subscribe to new properties")]
    Subscribe,
}

pub async fn run_bot(bot: Bot) -> anyhow::Result<()> {
    let state = Arc::new(BotContext::new(bot).await?);

    let message_handling_task = tokio::spawn(state.clone().message_task());
    let scraper_task = tokio::spawn(state.scraper_task());
    let (message_handling_result, scraper_result) =
        tokio::join!(message_handling_task, scraper_task);

    if let Err(e) = message_handling_result {
        tracing::error!("Message handling task failed: {:?}", e);
    }

    if let Err(e) = scraper_result {
        tracing::error!("Scraper task failed: {:?}", e);
    }

    Ok(())
}

#[derive(Clone)]
struct BotContext {
    persistence: Persistence,
    bot: Bot,
}

impl BotContext {
    async fn new(bot: Bot) -> anyhow::Result<Self> {
        Ok(Self {
            persistence: Persistence::new().await?,
            bot,
        })
    }

    async fn message_task(self: Arc<Self>) {
        let handler = Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command);

        Dispatcher::builder(self.bot.clone(), handler)
            .dependencies(dptree::deps![self.clone()])
            .build()
            .dispatch()
            .await;

        tracing::info!("Dispatcher exited");
    }

    async fn scraper_task(self: Arc<Self>) {
        loop {
            tracing::info!("Starting scrape");

            let pararius_scraper = ParariusScraper::default();
            if let Err(e) = self.scrape_once(pararius_scraper).await {
                tracing::error!("Scrape failed: {:?}", e);
            }

            tracing::info!("Sleeping for 5 minutes");
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    }

    async fn scrape_once<T: WebsiteScraper>(&self, scraper: T) -> anyhow::Result<()> {
        let existing_properties = self.persistence.list_properties().await?;

        let properties = scraper.list_properties().await?;

        let new_properties: Vec<_> = properties
            .into_iter()
            .filter(|property| !existing_properties.contains(&property.url))
            .collect();

        tracing::info!("Found {} new properties", new_properties.len());

        if new_properties.is_empty() {
            return Ok(());
        }

        let subscribers = self.persistence.list_subscribers().await?;
        if subscribers.is_empty() {
            return Ok(());
        }

        // Notify subscribers if there are relevant properties
        for property in new_properties.iter() {
            if property.price >= 1650 || property.area < 55 {
                continue;
            }

            // Sleep for a bit to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let full_property = scraper.scrape_property(property.clone()).await?;

            if !DESIRED_LOCATION.contains(&full_property.location) {
                continue;
            }

            for subscriber in subscribers.iter() {
                let chat_id = ChatId(*subscriber);
                if let Err(error) = self
                    .bot
                    .send_message(chat_id, format!("New property: {}", property.url))
                    .await
                {
                    tracing::error!("Failed to send subscriber notification: {:?}", error);
                }

                if let Err(error) = self
                    .bot
                    .send_location(
                        chat_id,
                        full_property.location.y(),
                        full_property.location.x(),
                    )
                    .await
                {
                    tracing::error!("Failed to send subscriber notification: {:?}", error);
                }
            }
        }

        // Save new properties to DB
        for property in new_properties.iter() {
            self.persistence.save_property(&property.url).await?;
        }

        Ok(())
    }

    async fn handle_message_inner(
        &self,
        bot: Bot,
        msg: Message,
        cmd: Command,
    ) -> Result<(), BotError> {
        match cmd {
            Command::Start => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?
            }
            Command::Subscribe => {
                self.persistence
                    .add_subscriber(msg.chat.id.0)
                    .await
                    .map_err(|_| BotError::Internal("failed to subscribe"))?;
                bot.send_message(msg.chat.id, "Subscribed!").await?
            }
        };

        Ok(())
    }

    async fn handle_message(&self, bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
        match self
            .handle_message_inner(bot.clone(), msg.clone(), cmd.clone())
            .await
        {
            Ok(()) => Ok(()),
            Err(BotError::Telegram(e)) => Err(e),
            Err(BotError::Internal(err)) => {
                bot.send_message(msg.chat.id, format!("Error: {:?}", err))
                    .await?;
                Ok(())
            }
        }
    }
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<BotContext>,
) -> ResponseResult<()> {
    state.handle_message(bot, msg, cmd).await
}

// Quick and dirty error handling for now
enum BotError {
    Internal(&'static str),
    Telegram(teloxide::RequestError),
}

impl From<teloxide::RequestError> for BotError {
    fn from(err: teloxide::RequestError) -> Self {
        Self::Telegram(err)
    }
}

mod persistence;
mod scraping;

use std::sync::Arc;

use persistence::Persistence;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let bot = Bot::from_env();
    let state = Arc::new(BotContext::new().await?);

    let message_handling_task = tokio::spawn(state.message_task(bot));
    message_handling_task.await?;



    Ok(())
}

#[derive(Clone)]
struct BotContext {
    persistence: Persistence,
}

impl BotContext {
    async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            persistence: Persistence::new().await?,
        })
    }

    async fn message_task(self: Arc<Self>, bot: Bot) {
        let handler = Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command);

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![self.clone()])
            .build()
            .dispatch()
            .await;
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

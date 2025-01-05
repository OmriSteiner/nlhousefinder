mod persistence;
mod scraping;

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
    let state = BotContext::new().await?;

    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[derive(Clone)]
struct BotContext;

impl BotContext {
    async fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    async fn handle_message(&self, bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
        match cmd {
            Command::Start => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?
            }
            Command::Subscribe => bot.send_message(msg.chat.id, "Subscribed!").await?,
        };

        Ok(())
    }
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: BotContext,
) -> ResponseResult<()> {
    state.handle_message(bot, msg, cmd).await
}

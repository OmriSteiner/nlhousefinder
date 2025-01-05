mod persistence;
mod scraping;

use teloxide::{prelude::*, utils::command::BotCommands};

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

    Command::repl(bot, handle_message).await;

    Ok(())
}

async fn handle_message(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Subscribe => bot.send_message(msg.chat.id, "Subscribed!").await?,
    };

    Ok(())
}

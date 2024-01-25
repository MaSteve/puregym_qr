use clap::Parser;
use puregym_qr::bot_lib::{get_dispatcher, get_webhook_listener, BotCredentials};
use std::sync::Arc;
use teloxide::prelude::*;

/// A simple Telegram bot for generating QR codes for accessing PureGym
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Get updates using webhooks instead of polling (default)
    #[arg(long, default_value_t = false)]
    webhook: bool,

    /// URL where the bot will get updates from the webhook
    #[arg(long, required_if_eq("webhook", "true"))]
    webhook_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    pretty_env_logger::init();

    log::info!("Fetching bot credentials.");
    let bot_credentials = BotCredentials::from_secrets()?;

    log::info!("Starting bot.");
    let bot = Bot::new(bot_credentials.bot_token);
    let chat_credentials = Arc::new(bot_credentials.chat_credentials);
    let mut dispatcher = get_dispatcher(bot.clone(), chat_credentials);

    if args.webhook {
        dispatcher
            .dispatch_with_listener(
                get_webhook_listener(bot.clone(), args.webhook_url.unwrap()).await?,
                LoggingErrorHandler::with_custom_text("Error from the update listener"),
            )
            .await;
    } else {
        dispatcher.dispatch().await;
    }

    Ok(())
}

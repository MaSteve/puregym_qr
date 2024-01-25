use clap::Parser;
use puregym_qr::bot_lib::BotCredentials;
use teloxide::prelude::*;

/// Telegram bot webhook management for AWS lambdas
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Enable webhook
    #[arg(long, default_value_t = false)]
    enable: bool,

    /// Disable webhook
    #[arg(long, default_value_t = true)]
    disable: bool,

    /// URL where the bot will get updates from the webhook
    #[arg(long, required_if_eq("enable", "true"))]
    webhook_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    pretty_env_logger::init();
    log::info!("Fetching bot credentials.");
    let bot_credentials = BotCredentials::from_secrets()?;
    log::info!("Initialize bot.");
    let bot = Bot::new(bot_credentials.bot_token);
    if args.enable {
        log::info!("Enabling webhook.");
        let url = args
            .webhook_url
            .unwrap()
            .parse()
            .expect("Unparsable webhook URL.");
        bot.set_webhook(url).send().await?;
    } else {
        log::info!("Disabling webhook.");
        bot.delete_webhook().send().await?;
    }
    Ok(())
}

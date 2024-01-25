use crate::api::{generate_qr, LoginCredentials};
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::sync::Arc;
use teloxide::dispatching::DefaultKey;
use teloxide::dispatching::UpdateHandler;
use teloxide::update_listeners::UpdateListener;
use teloxide::{
    prelude::*, types::InputFile, update_listeners::webhooks, utils::command::BotCommands,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
enum Command {
    QR,
}

async fn process_commands(
    bot: Bot,
    chat_credentials: Arc<LoginCredentialsMap>,
    msg: Message,
    cmd: Command,
) -> Result<(), anyhow::Error> {
    log::info!("Command {:?} received from chat {}", cmd, msg.chat.id);
    if let Some(login_credentials) = chat_credentials.get(&msg.chat.id.0) {
        log::info!("Processing command {:?} from chat {}", cmd, msg.chat.id);
        match cmd {
            Command::QR => {
                bot.send_message(msg.chat.id, "Calling API...").await?;
                let qr_code = generate_qr(login_credentials)?;
                bot.send_photo(msg.chat.id, InputFile::read(Cursor::new(qr_code)))
                    .await?;
            }
        }
    } else {
        log::info!("Ignoring command {:?} from chat {}", cmd, msg.chat.id);
    }
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum BotSetupError {
    #[error(transparent)]
    SetupError(#[from] anyhow::Error),
}

pub type LoginCredentialsMap = HashMap<i64, LoginCredentials>;

#[derive(Deserialize)]
pub struct BotCredentials {
    pub chat_credentials: LoginCredentialsMap,
    pub bot_token: String,
}

impl BotCredentials {
    pub fn from_secrets() -> Result<Self, BotSetupError> {
        let secrets_path = "secrets.json";
        let mut file = File::open(secrets_path).context("Failed to open secrets file.")?;

        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .context("Failed to read secrets file.")?;

        Ok(serde_json::from_str(&file_content).context("Failed to deserialize secrets.")?)
    }
}

pub fn get_update_handler() -> UpdateHandler<anyhow::Error> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(process_commands)
}

pub fn get_dispatcher(
    bot: Bot,
    chat_credentials: Arc<LoginCredentialsMap>,
) -> Dispatcher<Bot, anyhow::Error, DefaultKey> {
    Dispatcher::builder(bot, get_update_handler())
        .dependencies(dptree::deps![chat_credentials])
        .enable_ctrlc_handler()
        .build()
}

pub async fn get_webhook_listener(
    bot: Bot,
    webhook_url: String,
) -> Result<impl UpdateListener<Err = std::convert::Infallible>, BotSetupError> {
    let addr = ([0, 0, 0, 0], 8443).into();
    let url = webhook_url
        .parse()
        .context("Failed to parse webhook URL.")?;
    Ok(
        webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
            .await
            .context("Failed to setup webhook listener.")?,
    )
}

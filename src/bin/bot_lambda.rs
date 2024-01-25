use lambda_http::{
    http::StatusCode, run, service_fn, Error, IntoResponse, Request, RequestPayloadExt,
};
use puregym_qr::bot_lib::{get_update_handler, BotCredentials};
use std::{ops::Deref, sync::Arc};
use teloxide::{
    dispatching::UpdateHandler,
    prelude::{ControlFlow, DependencyMap, Request as TeloxideRequest, Requester},
    types::Update,
    Bot,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();
    log::info!("Fetching bot credentials.");
    let bot_credentials = BotCredentials::from_secrets()?;
    let chat_credentials = Arc::new(bot_credentials.chat_credentials);

    log::info!("Starting bot.");
    let bot = Bot::new(bot_credentials.bot_token);

    log::info!("Initializing handler dependencies.");
    let mut deps = DependencyMap::new();
    let me = bot.get_me().send().await?;
    deps.insert(me);
    deps.insert(chat_credentials);
    deps.insert(bot);
    let deps = Arc::new(deps);

    let handler = get_update_handler();

    run(service_fn(|event: Request| {
        function_handler(&deps, &handler, event)
    }))
    .await
}

pub async fn function_handler(
    deps: &Arc<DependencyMap>,
    handler: &UpdateHandler<anyhow::Error>,
    event: Request,
) -> Result<impl IntoResponse, Error> {
    let update = event.payload::<Update>()?.unwrap();
    log::info!("Update received: {:?}", update);

    let mut deps = deps.deref().clone();
    deps.insert(update);

    match handler.dispatch(deps).await {
        ControlFlow::Break(Ok(())) => {}
        ControlFlow::Break(Err(err)) => {
            log::error!("Error from the UpdateHandler: {:?}.", err);
        }
        ControlFlow::Continue(_) => {
            log::warn!("Update was not handled by bot.");
        }
    }

    Ok((StatusCode::OK, ""))
}

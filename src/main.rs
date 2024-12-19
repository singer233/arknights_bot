
mod bot_core;
mod callback_handle;
mod utils;
mod config_builder;

use teloxide::dispatching::DefaultKey;
use teloxide::prelude::*;
use crate::bot_core::init_bot;
use crate::config_builder::get_config;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init().await
        .dispatch()
        .await;
}
async fn init()->Dispatcher<Bot, anyhow::Error, DefaultKey>{
    let config = get_config();
    let bot_config = config.config();
    init_bot(bot_config)
}
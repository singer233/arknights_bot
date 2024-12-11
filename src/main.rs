
mod bot_core;
mod callback_handler;
mod callback_manager;
mod utils;

use teloxide::prelude::*;
#[tokio::main]
async fn main() {
    let mut bot = bot_core::build_bot_;
    bot.dispatch().await;
}
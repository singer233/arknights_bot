
mod bot_core;
mod callback_handler;
mod callback_manager;

use teloxide::prelude::*;
#[tokio::main]
async fn main() {
    let mut bot = bot_core::build_bot("6424873633:AAE4GJ71jg6vUmhIGnEpr7iSOwhCIAyyTtQ");
    bot.dispatch().await;
}
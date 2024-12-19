use std::sync::OnceLock;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use teloxide::dispatching::{DefaultKey, DpHandlerDescription};
use teloxide::prelude::*;
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;
use crate::callback_handle;

pub static THE_BOT: OnceLock<TheBot> = OnceLock::new();
#[derive(Deserialize,Serialize,Debug)]
pub struct BotConfig {
    token: String,
    owner_id : i64,
    callback_timeout_sec: u64
}
pub struct TheBot {
    callback_manager: crate::callback_handle::callback_manager::CallbackManager,
    cancellation_token: CancellationToken,
    bot: Bot
}

impl TheBot {
    pub fn get_bot(&self) -> Bot {
        self.bot.clone()
    }
    pub fn get_callback_manager(&self) -> &callback_handle::callback_manager::CallbackManager {
        &self.callback_manager
    }
}
impl From<BotConfig> for TheBot {
    fn from(config: BotConfig) -> Self {
        let bot = Bot::new(config.token);
        let cancellation_token = CancellationToken::new();
        let callback_manager = crate::callback_handle::callback_manager::CallbackManager::new(cancellation_token.clone(),Duration::from_secs(config.callback_timeout_sec),bot.clone());
        TheBot {
            callback_manager,
            cancellation_token,
            bot
        }
    }
}

pub fn init_bot(bot_config : BotConfig) ->Dispatcher<Bot, anyhow::Error, DefaultKey>{
    THE_BOT.get_or_init(|| TheBot::from(bot_config));
    build_bot(&THE_BOT.get().unwrap().bot)
}
pub(crate) fn build_bot(bot: &Bot) -> Dispatcher<Bot, anyhow::Error, DefaultKey> {
    let message_schema = Update::filter_message().endpoint(message_handler);
    let member_schema = Update::filter_chat_member();
    let inline_query_schema = Update::filter_inline_query();
    let callback_query_schema = Update::filter_callback_query().endpoint(callback_handle::global_callback_handler);
    let schema = message_schema.chain(member_schema).chain(inline_query_schema)
        .chain(callback_query_schema)
    ;
    Dispatcher::builder(bot.clone(),schema).build()
}
async fn message_handler(bot: Bot,message: Message)->anyhow::Result<()> {
    bot.send_message(message.chat.id,format!("message:{:?}",message)).await?;
    Ok(())
}
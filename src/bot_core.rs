use teloxide::dispatching::{DefaultKey, DpHandlerDescription};
use teloxide::prelude::*;
use tokio::io::AsyncReadExt;
use crate::callback_handler;

pub(crate) fn build_bot() -> Dispatcher<Bot, anyhow::Error, DefaultKey> {
    let bot = Bot::from_env();
    let message_schema = Update::filter_message().endpoint(message_handler);
    let member_schema = Update::filter_chat_member();
    let inline_query_schema = Update::filter_inline_query();
    //let callback_query_schema = Update::filter_callback_query().endpoint(callback_handler::global_callback_handler);
    let schema = message_schema.chain(member_schema).chain(inline_query_schema)
        //.chain(callback_query_schema)
    ;
    return Dispatcher::builder(bot,schema).build()
}
async fn message_handler(bot: Bot,message: Message)->anyhow::Result<()> {
    bot.send_message(message.chat.id,format!("message:{:?}",message)).await?;
    Ok(())
}
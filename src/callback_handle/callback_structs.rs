use teloxide::prelude::*;
use teloxide::types::{Chat, MaybeInaccessibleMessage, MessageId, User};
use thiserror::Error;
#[derive(Error, Debug)]
pub enum CallbackError {
    #[error("can not parse input uuid")]
    UUidParseError(#[from] uuid::Error),
    #[error("callback function failed")]
    CallbackFunctionError(#[from] anyhow::Error),
    #[error("uuid not found in hashmap")]
    CallbackNotFound,
    #[error("invalid callback data")]
    InvalidCallbackData,
}
#[async_trait::async_trait]
pub trait CallbackEntity :Send {
    async fn callback_ex(&mut self, bot: Bot, callback_query: CallbackQuery) -> anyhow::Result<bool>{
        let callback_user = callback_query.from;
        let (callback_chat,callback_msg_id) = callback_query.message.map(|x| match x {
            MaybeInaccessibleMessage::Inaccessible(k) => {(k.chat,k.message_id)}
            MaybeInaccessibleMessage::Regular(k) => {(k.chat,k.id)}
        }).unzip();
        let mut callback_data = callback_query.data.unwrap();
        let mut callback_data = callback_data.split(",");
        callback_data.next();
        self.callback(bot,callback_data.map(|s| s.to_string()).collect(),callback_user,callback_chat,callback_msg_id).await

    }
    async fn callback(&mut self, bot: Bot, args : Vec<String>,user : User,chat :Option<Chat>,message_id:Option<MessageId>) -> anyhow::Result<bool>;
    async fn on_timeout(&mut self,bot: Bot)->anyhow::Result<()>{Ok(())}
}
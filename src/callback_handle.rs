use std::error::Error;
use anyhow::bail;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use uuid::Uuid;
use callback_structs::CallbackError;
use crate::bot_core::THE_BOT;

pub mod callback_structs;
pub mod callback_manager;
pub(crate) async fn global_callback_handler(bot: Bot, callback_query: CallbackQuery)->anyhow::Result<()> {
    bot.send_message(callback_query.chat_id().unwrap(),"Callback").send().await?;
    let uuid = callback_query.data.as_ref()
        .map(|x| x.split(",").next())
        .flatten()
        .map(|x| Uuid::parse_str(x))
        .transpose();
    match uuid {
        Ok(Some(uuid)) => {
            match THE_BOT.get().unwrap().get_callback_manager().run_callback(uuid, bot, callback_query).await{
                Err(e) => {
                    bail!(CallbackError::CallbackFunctionError(e));
                }
                _ =>{Ok(())}
            }

        },
        Ok(None) => {
            bail!(CallbackError::InvalidCallbackData)
        },
        Err(e) => {
            bail!(CallbackError::UUidParseError(e));
        }
    }
}
#[cfg(test)]
mod test {
    use teloxide::payloads::SendMessage;
    use teloxide::types::{Chat, InlineKeyboardButton, MaybeInaccessibleMessage, MessageId, User};
    use crate::bot_core::{init_bot, TheBot, THE_BOT};
    use crate::callback_handle::callback_manager::CallbackManager;
    use crate::config_builder::get_config;
    use super::*;
    struct TestCallbackEntity;
    #[async_trait::async_trait]
    impl callback_structs::CallbackEntity for TestCallbackEntity {
        async fn callback(&mut self, bot: Bot, args: Vec<String>, user: User, chat: Option<Chat>, message_id: Option<MessageId>) -> anyhow::Result<bool> {
            log::info!("callback");
            bot.edit_message_text(chat.unwrap().id,message_id.unwrap(),format!("callback:{:?}",args)).send().await?;
            if args[0]=="F" {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        async fn on_timeout(&mut self, bot: Bot) -> anyhow::Result<()> {
            log::info!("timeout");
            Ok(())
        }
    }
    async fn callback_message_handler(bot: Bot,message: Message)->anyhow::Result<()> {
        let uuid = THE_BOT.get().unwrap().get_callback_manager().add_callback(Box::new(TestCallbackEntity)).await;

        let inline_keyboard = vec![vec![("C","F"),("T","T")]];

        let msg = bot.send_message(message.chat.id,format!("message:{:?}",message))
            .reply_markup(CallbackManager::inline_button_maker(uuid,inline_keyboard));
        msg.send().await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_callback(){
        THE_BOT.get_or_init(|| TheBot::from(get_config().config()));
        let message_schema = Update::filter_message().endpoint(callback_message_handler);
        let callback_schema = Update::filter_callback_query().endpoint(global_callback_handler);
        Dispatcher::builder(THE_BOT.get().unwrap().get_bot(), dptree::entry().branch(callback_schema).branch(message_schema)).build().dispatch().await;

    }
}
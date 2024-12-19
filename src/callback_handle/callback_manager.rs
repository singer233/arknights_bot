use std::collections::HashMap;
use std::time::Duration;
use anyhow::bail;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use toml::value::Date;
use uuid::Uuid;
use crate::bot_core::{THE_BOT};
use crate::callback_handle::callback_structs::{CallbackEntity, CallbackError};
use crate::callback_handle::callback_structs::CallbackError::CallbackFunctionError;
use crate::utils::{ObjectStorage, TimedFunction};

pub struct CallbackManager {
    callbacks: Mutex<ObjectStorage<(CancellationToken,Box<dyn CallbackEntity>)>>,
    cancellation_token: CancellationToken,
    default_callback_duration: Duration,
    bot: Bot,
}
impl CallbackManager {
    pub(crate) fn new(cancellation_token: CancellationToken, default_duration:Duration, bot:Bot) -> Self {
        CallbackManager {
            callbacks: Mutex::new(ObjectStorage::new()),
            cancellation_token,
            default_callback_duration: default_duration,
            bot,
        }
    }
    #[must_use]
    pub(super) async fn add_callback(&self, callback: Box<dyn CallbackEntity>) -> Uuid {
        let sub_token = self.cancellation_token.child_token();
        let mut lock = self.callbacks.lock().await;
        let uuid = lock.insert((sub_token.clone(),callback));
        {
            let uuid = uuid.clone();
            TimedFunction::function_with_custom_token(async move {
                THE_BOT.get().unwrap().get_callback_manager().on_timout(uuid).await;
            },self.default_callback_duration,sub_token);
        }
        uuid
    }
    pub(super) async fn run_callback(&self,uuid: Uuid,bot: Bot,callback_query: CallbackQuery) ->anyhow::Result<()> {
        let mut lock = self.callbacks.lock().await;
        let mut is_finish = false;
        let mut result = Ok(());
        if let Some((token,callback)) = lock.get_mut(&uuid) {
            match callback.callback_ex(bot,callback_query).await{
                Ok(finish) => {
                    if finish {
                        token.cancel();
                        is_finish = true;
                    }
                }
                Err(e) => {
                    is_finish = true;
                    log::warn!("Error while executing callback: {:?}",e);
                    result = Err(e)
                }
            }
        } else {
            bail!(CallbackError::CallbackNotFound);
        }
        if is_finish {
            lock.remove(&uuid);
        }
        if let Err(e) = result {
            bail!(e);
        }
        Ok(())

    }

    async fn on_timout(&self,uuid:Uuid) {
        let mut lock = self.callbacks.lock().await;
        if let Some((_,mut callback)) = lock.remove(&uuid) {
            match callback.on_timeout(self.bot.clone()).await{
                Ok(_) => {}
                Err(e) => {
                    log::warn!("Error while executing callback: {:?}",e);
                }
            }
        }

    }

    #[inline]
    pub fn callback_data_warp<S : Into<String>>(uuid : Uuid,data: S)->String{
        format!("{},{}",uuid.to_string(),data.into())
    }
    fn callback_warp_maker<S : Into<String>>(uuid : Uuid) -> impl Fn(S) -> String {
        move |x : S| format!("{},{}",uuid.to_string(),x.into())
    }
    pub fn inline_button_maker<T,C>(uuid:Uuid,data : Vec<Vec<(T,C)>>) -> InlineKeyboardMarkup
    where
        T: Into<String>,
        C: Into<String>
    {
        let button :Vec<Vec<InlineKeyboardButton>>= data.into_iter().map(
            |x| x.into_iter().map(
                |(t,c)| InlineKeyboardButton::callback(t.into(),CallbackManager::callback_data_warp(uuid,c))
            ).collect()
        ).collect();
        InlineKeyboardMarkup::new(button)
    }
}
#[cfg(test)]
mod test{
  use super::*;

}

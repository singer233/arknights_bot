use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use teloxide::prelude::*;


pub(crate) async fn global_callback_handler(bot: Bot, callback_query: CallbackQuery) -> anyhow::Result<()> {
    todo!("")
}
#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_callback(){

    }
}
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use teloxide::prelude::*;

/*type CallbackClosure = dyn FnOnce(Bot, Vec<dyn TryFrom<String, Error=Box<dyn Error>>>,CallbackQuery) -> (bool,anyhow::Result<()>);
static CALLBACKS: Mutex<HashMap<String, Box<CallbackClosure>>> = Mutex::new(HashMap::new());
pub(crate) async fn global_callback_handler(bot: Bot, callback_query: CallbackQuery) -> anyhow::Result<()> {
    let data = (&callback_query.data).as_ref();
    let user = &callback_query.from;
    if let Some(data) = data{
        let mut callback_args = data.split(",");
        let callback_name =  callback_args.next();
        if let Some(callback_name) = callback_name{

        }



    } else {
        // 无回调数据这很奇怪
        callback_error(bot,callback_query).await
    }
}
async fn callback_error(bot :Bot,callback_query: CallbackQuery) -> anyhow::Result<()> {
    bot.send_message(callback_query.from.id, "回调数据错误").await?;
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_callback(){

    }
}*/
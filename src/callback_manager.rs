use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::Duration;
use crate::utils::ObjectStorage;
use crate::utils::TimedFunction;
use teloxide::Bot;
use teloxide::prelude::*;
use uuid::Uuid;
use anyhow::Result;
use futures::future::BoxFuture;
use log::error;
use tokio_util::sync::CancellationToken;

type CallbackFunc<'a> = fn(Bot, CallbackQuery, Option<Box<dyn SendAny>>) -> BoxFuture<'static,Result<Option<Box<dyn SendAny>>>>;
type TimeoutFunc<'a> = fn(Bot,Option<Box<dyn SendAny>>)-> BoxFuture<'static,Result<()>>;
trait SendAny: Send + Any {}
impl<T: Send + Any> SendAny for T {}
struct CallBackContext {
    custom_context : Option<Box<dyn SendAny>>,
    callback: CallbackFunc<'static>
}
impl CallBackContext {
    fn new<T:SendAny>(context: Option<T>,callback: CallbackFunc)-> Self{
        CallBackContext{
            custom_context: context.map(|x| Box::new(x) as Box<dyn SendAny>),
            callback
        }
    }
    async fn run(&mut self,bot: Bot, callback_query: CallbackQuery)->Result<Option<Box<dyn SendAny>>>{
        let context = self.custom_context.take();
        (self.callback)(bot,callback_query,context).await
    }
}
static CALLBACK_CONTEXT_MAP: LazyLock<tokio::sync::Mutex<ObjectStorage<CallBackContext>>> = LazyLock::new(|| tokio::sync::Mutex::new(ObjectStorage::new()));
static CALLBACK_CLEANUP: OnceLock<Mutex<TimedFunction>> = OnceLock::new();
const SESSION_TIMEOUT: u64 = 60;
pub fn init_callback_query(shutdown_token :&CancellationToken){
    CALLBACK_CLEANUP.set(Mutex::new(TimedFunction::new(shutdown_token.child_token()))).unwrap();
}
async fn register_callback_ex<T:SendAny>(callback_function: CallbackFunc<'_>,context: T,on_timeout : TimeoutFunc<'_>,bot: Bot)->Uuid{
    let mut locked_map  = CALLBACK_CONTEXT_MAP.lock().await;
    let uuid = locked_map.insert(CallBackContext::new(Some(context),callback_function));
    CALLBACK_CLEANUP.get().unwrap().lock().unwrap().add_function_with_uuid(
        async move {
            let timeout = on_timeout;
            let new_context= CALLBACK_CONTEXT_MAP.lock().await.remove(&uuid)
                .and_then(|x| x.custom_context);
            if (timeout(bot,new_context).await).is_err(){
                error!("Error in on_timeout function");
            }
        },
        Duration::from_secs(60),
        &uuid
    ).unwrap();
    uuid
}

async fn run_callback(uuid: &Uuid,bot: Bot, callback_query: CallbackQuery){
    let mut callback_lock = CALLBACK_CONTEXT_MAP.lock().await;
    let mut callback = callback_lock.get_mut(uuid);
    let callback_return;
    let mut need_cleanup = false;
    if let Some(callback) = callback {
        callback_return = callback.run(bot,callback_query).await;
        match callback_return {
            Ok(Some(context)) => {
                callback.custom_context = Some(context);
            }
            Ok(None) => {
                need_cleanup = true;
            }
            Err(e) => {
                error!("Error in callback function: {:?}",e);
                need_cleanup = true;
            }
        }
    } else {
        return;
    }
    if need_cleanup{
        callback_lock.remove(uuid);
    }
    return;
}
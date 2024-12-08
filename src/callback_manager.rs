use std::any::Any;
use std::collections::HashMap;
use teloxide::Bot;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use uuid::Uuid;
use anyhow::Result;
mod timer_struct;
struct Context {
    custom_context : Box<dyn Any>,
    callback: fn(Bot,CallbackQuery,Box<dyn Any>)->Result<()>
}
static CALLBACK_CONTEXT_MAP: Mutex<HashMap<String,Context>> =Mutex::new(HashMap::new());
fn register_callback(target_user:UserId)->Uuid{
    todo!("")
}
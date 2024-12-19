use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use anyhow::ensure;
use log::trace;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use crate::utils::uuid_object_storage::ObjectStorage;

#[derive(Debug)]
pub struct TimedFunction{
    cancel_token_map: Arc<RwLock<ObjectStorage<CancellationToken>>>,
    main_token: CancellationToken
}
impl TimedFunction{
    pub(crate) fn new(cancellation_token: CancellationToken) ->Self{
        TimedFunction{
            cancel_token_map: Arc::new(RwLock::new(ObjectStorage::new())),
            main_token: cancellation_token
        }
    }
    pub(crate) fn add_function<F>(&mut self, future: F, delay:Duration) ->Uuid
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let token = self.main_token.child_token();
        let uuid = self.cancel_token_map.write().unwrap().insert(token.clone());
        {
            let map = self.cancel_token_map.clone();
            let uuid = uuid.clone();
            tokio::spawn(
                async move {
                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {
                            future.await;
                            map.write().unwrap().remove(&uuid);
                        }
                        _ = token.cancelled() => {
                            // do nothing
                        }
                    }
                }
            );
        }
        uuid

    }
    #[must_use]
    #[inline]
    pub(crate) fn add_function_with_token<F>(&mut self, future: F, delay:Duration) ->CancellationToken
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let token = self.main_token.child_token();
        {
            let token = token.clone();
            Self::lunch_function(future,delay,token);
        }
        token
    }
    pub(crate) fn cancel_function(&mut self, uuid: &Uuid){
        if let Some(token) = self.cancel_token_map.write().unwrap().remove(uuid){
            token.cancel();
        } else {
            trace!("cancel_function: uuid not found");
        }
    }
    #[inline]
    pub(crate) fn function_with_custom_token<F>(future: F, delay:Duration, sub_token:CancellationToken)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        Self::lunch_function(future,delay,sub_token);
    }

    fn lunch_function<F>(future: F, delay:Duration,token:CancellationToken)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(
            async move {
                tokio::select! {
                        _ = tokio::time::sleep(delay) => {
                            future.await;
                        }
                        _ = token.cancelled() => {
                            // do nothing
                        }
                    }
            }
        );
    }
}
#[cfg(test)]
mod test{
    use super::*;
    use tokio::time::Duration;
    #[tokio::test]
    async fn test_just_run(){
        let token = CancellationToken::new();
        let mut timed_function = TimedFunction::new(token.clone());
        let p = Arc::new(Mutex::new(0));
        {
            let p = p.clone();
            timed_function.add_function(async move{
                *(p.lock().unwrap()) = 3;
            },Duration::from_secs(1));
        }
        tokio::time::sleep(Duration::from_secs_f32(1.1)).await;
        let k = p.lock().unwrap().clone();
        assert_eq!(k, 3);

    }
    #[tokio::test]
    async fn test_uuid_cancel(){
        let token = CancellationToken::new();
        let mut uuid :Uuid;
        let mut timed_function = TimedFunction::new(token.clone());
        let p = Arc::new(Mutex::new(0));
        {
            let p = p.clone();
            uuid = timed_function.add_function(async move{
                *(p.lock().unwrap()) = 3;
            },Duration::from_secs(3));
        }
        tokio::time::sleep(Duration::from_secs_f32(1.1)).await;
        timed_function.cancel_function(&uuid);
        let k = p.lock().unwrap().clone();
        assert_eq!(k, 0);

    }
    #[tokio::test]
    async fn test_cancel_all(){
        let token = CancellationToken::new();
        let mut timed_function = TimedFunction::new(token.clone());
        let p = Arc::new(Mutex::new(0));
        {
            let p = p.clone();
            timed_function.add_function(async move{
                *(p.lock().unwrap()) = 3;
            },Duration::from_secs(3));
        }
        {
            let p = p.clone();
            timed_function.add_function(async move{
                *(p.lock().unwrap()) = 10;
            },Duration::from_secs(9));
        }
        tokio::time::sleep(Duration::from_secs_f32(1.1)).await;
        token.cancel();
        let k = p.lock().unwrap().clone();
        assert_eq!(k, 0);

    }
}

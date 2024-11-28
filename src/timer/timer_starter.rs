use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use log::{error, info, log};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;
use crate::timer::timed_manager::TimedManager;
type AsyncBlock = Pin<Box<dyn Future<Output = ()>>>;
enum TimerMessage {
    Add((Box<dyn Future<Output = ()>>,u64)),
    Remove(Uuid),
    Stop
}

pub(crate) async fn timer_future(mut rx: Receiver<TimerMessage>, tx :Sender<anyhow::Result<Uuid>>){
    let mut timer_manager = TimedManager::new();
    let mut delayed_duration = timer_manager.get_next_delay();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Some(msg) = msg{
                    match msg{
                        TimerMessage::Add((future,delay)) => {
                            let uuid = timer_manager.add(Box::pin(future),delay);
                            if tx.send(uuid).await.is_err(){
                                error!("Failed to send uuid to channel shutting down");
                                break;
                            }
                        }
                        TimerMessage::Remove(uuid) => {
                            timer_manager.remove(uuid);
                        }
                        TimerMessage::Stop => {
                            info!("Stop Msg Recived shutting down");
                            return;
                        }
                    }
                } else {
                    error!("Timer channel closed unexpectedly shutting down");
                    break;
                }
            }
            _ = tokio::time::sleep(delayed_duration) => {
                timer_manager.execute().await;
            }
        }

    }
}
// TODO : Test this code
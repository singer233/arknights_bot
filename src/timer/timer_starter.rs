use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use log::{error, info, log};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;
type AsyncBlock = Pin<Box<dyn Future<Output = ()>>>;
enum TimerMessage {
    Add((Box<dyn Future<Output = ()>>,u64)),
    Remove(Uuid),
    Stop
}

pub(crate) async fn timer_future(mut rx: Receiver<TimerMessage>, tx :Sender<Uuid>){
    let mut delayed_duration = Duration::MAX;
    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Some(msg) = msg{
                    match msg{
                        TimerMessage::Add((future,delay)) => {

                        }
                        TimerMessage::Remove(uuid) => {
                            
                        }
                        TimerMessage::Stop => {
                            info!("Stop Msg Recived shutting down");
                            break;
                        }
                    }
                } else {
                    error!("Timer channel closed unexpectedly shutting down");
                    break;
                }
            }
            _ = tokio::time::sleep(delayed_duration) => {

            }
        }

    }
}
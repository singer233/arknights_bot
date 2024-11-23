use std::cmp::Ordering;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use crate::timer::timed_function::TimedFunction;

pub(crate) async fn timer_future(rx : Receiver<Option<TimedFunction>>){
    loop {

    }
}
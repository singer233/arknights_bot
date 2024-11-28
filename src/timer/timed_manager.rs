use std::cmp::{Ordering, Reverse};
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::time;
use std::time::Duration;
use log::{error, trace, warn};
use uuid::Uuid;

type AsyncBlock = Pin<Box<dyn Future<Output = ()>>>;
pub(super) struct ComparableUUid {
    target_time: Reverse<u64>,
    uuid: Uuid,
}

impl Eq for ComparableUUid {}

impl PartialEq<Self> for ComparableUUid {
    fn eq(&self, other: &Self) -> bool {
        self.uuid==other.uuid
    }
}

impl PartialOrd<Self> for ComparableUUid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.target_time.partial_cmp(&other.target_time)
    }
}

impl Ord for ComparableUUid{
    fn cmp(&self, other: &Self) -> Ordering {
        self.target_time.cmp(&other.target_time)
    }
}
pub(super) struct TimedManager{
    function_map: std::collections::HashMap<Uuid,AsyncBlock>,
    heap: std::collections::BinaryHeap<ComparableUUid>,
    next_exec : Option<(Uuid,AsyncBlock)>,
    next_timestamp: Option<u64>
}
impl TimedManager{
    pub(super) fn new() ->Self{
        TimedManager{
            function_map: std::collections::HashMap::new(),
            heap: std::collections::BinaryHeap::new(),
            next_exec: None,
            next_timestamp: None
        }
    }
    pub(super) fn add(&mut self, future: AsyncBlock, delay: u64)->anyhow::Result<Uuid>{
        let mut uuid = uuid::Uuid::new_v4();
        while self.function_map.contains_key(&uuid){
            uuid = uuid::Uuid::new_v4();
        }
        let current_timestamp = Self::get_unix_timestamp()?;
        self.function_map.insert(uuid,future);
        self.heap.push(ComparableUUid{
            uuid,
            target_time: Reverse(current_timestamp + delay)
        });
        Ok(uuid)
    }
    pub(super) fn get_next_delay(&self)->Duration{
        if let Some(next_timestamp) = self.next_timestamp {
            if let Ok(current_timestamp) = Self::get_unix_timestamp(){
                if next_timestamp > current_timestamp{
                    Duration::from_secs(next_timestamp - current_timestamp)
                } else {
                    warn!("Next timestamp is in the past this should not happen \n
                     Maybe some operation take too long to execute");
                    Duration::from_secs(0)
                }
            } else {
                error!("Failed to get current timestamp this should not happen");
                Duration::MAX
            }
        } else {
            Duration::MAX
        }
    }
    pub(super) fn remove(&mut self, uuid: Uuid){
        if let Some(next_exec) = self.next_exec.take(){
            if next_exec.0 == uuid{
                self.next_exec = None;
                self.load_next_future();
            }
        } else {
            if self.function_map.remove(&uuid).is_none(){
                warn!("Trying to remove non-exist future");
            }
        }
    }
    pub(super) async fn execute(&mut self){
        self.real_execute().await;
        self.load_next_future();
        while let Some(next_timestamp) = self.next_timestamp{
                if let Ok(current_timestamp) = Self::get_unix_timestamp(){
                    if next_timestamp <= current_timestamp{
                        self.real_execute().await;
                        self.load_next_future();
                    } else {
                        return;
                    }
                } else {
                    error!("Failed to get current timestamp");
                }
        }

    }
    fn load_next_future(&mut self){
        while let Some(mut next) = self.heap.pop(){
                if self.function_map.contains_key(&next.uuid){
                    self.next_exec = Some((next.uuid,self.function_map.remove(&next.uuid).unwrap()));
                    self.next_timestamp = Some(next.target_time.0);
                    break;
                } else {
                    trace!("Future with uuid {} was removed",next.uuid);
                    continue;
                }
            }
        }
    #[inline]
    async fn real_execute(&mut self){
        if let Some(future) = self.next_exec.take(){
            future.1.await;
            self.next_timestamp = None;
        }
    }
    #[inline]
    fn get_unix_timestamp()->anyhow::Result<u64>{
        Ok(time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs())
    }
}
// TODO : Test this code
use std::cmp::{Ordering, Reverse};
use std::future::Future;
use std::pin::Pin;
use std::time;
use log::warn;
use uuid::Uuid;

type AsyncBlock = Pin<Box<dyn Future<Output = ()>>>;
struct ComparableUUid {
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
    next_exec : Option<(Uuid,AsyncBlock)>
}
impl TimedManager{
    pub(super) fn new() ->Self{
        TimedManager{
            function_map: std::collections::HashMap::new(),
            heap: std::collections::BinaryHeap::new(),
            next_exec: None,
        }
    }
    pub(super) fn add(&mut self, future: AsyncBlock, delay: u64)->uuid::Uuid{
        let mut uuid = uuid::Uuid::new_v4();
        while self.function_map.contains_key(&uuid){
            uuid = uuid::Uuid::new_v4();
        }
        let current_timestamp = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        self.function_map.insert(uuid,future);
        self.heap.push(ComparableUUid{
            uuid,
            target_time: Reverse(current_timestamp + delay)
        });
        uuid
    }
    pub(super) fn remove(&mut self, uuid: uuid::Uuid){
        if let Some(next_exec) = self.next_exec.take(){
            if next_exec.0 == uuid{
                self.next_exec = None;
            }
        } else {
            if self.function_map.remove(&uuid).is_none(){
                warn!("Trying to remove non-exist future");
            }
        }
    }
    pub(super) async fn execute(&mut self){
        self.real_execute().await;
        while let Some(mut next) = self.heap.pop(){
            todo!("Implement the logic to execute the future");
        }
    }
    #[inline]
    async fn real_execute(&mut self){
        if let Some(future) = self.next_exec.take(){
            future.1.await;
        }
    }

}
use std::cmp::{Ordering, Reverse};
use std::future::Future;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
type AsyncFn = Pin<Box<dyn Future<Output = ()>>>;
pub(crate) struct TimedFunction {
    delay_to : Reverse<u64>,
    function: AsyncFn
}
impl TimedFunction{
    pub fn new(delay_sec: u64, function: AsyncFn) -> Self {
        let unix_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Self {
            delay_to :Reverse(delay_sec+unix_timestamp.as_secs()),
            function
        }
    }
    async fn exec(self){
        self.function.await;
    }
}

impl Eq for TimedFunction {}

impl PartialEq<Self> for TimedFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl PartialOrd<Self> for TimedFunction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.delay_to.partial_cmp(&other.delay_to)
    }
}

impl Ord for TimedFunction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.delay_to.cmp(&other.delay_to)
    }
}
mod test {
    use std::pin::pin;
    use super::*;

    #[tokio::test]
    async fn timed_function() {
        let k   = async { dummy_function(5).await};
        let timed_function = TimedFunction::new(5, Box::pin(k));
        let timed_function2 = TimedFunction::new(6, Box::pin(async {dummy_function(6).await }));
        assert!(timed_function==timed_function);
        assert!(timed_function > timed_function2);
        timed_function.exec().await;

    }
    async fn dummy_function(i: u64) {
        println!("Hello World {}",i);
    }

}
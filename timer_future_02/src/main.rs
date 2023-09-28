use std::time::Duration;
use futures::executor::block_on;
use timer_future_02::TimerFuture;

fn main() {
    let future = TimerFuture::new(Duration::new(3, 0));
    block_on(future);
}

use std::{ 
    future::Future, 
    pin:: Pin, 
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

/*
    TimerFuture 让线程来传达定时器的时间已经到了，这个 Future 可以完成了
*/
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

// 在 Future 和等待的线程之间共享的状态
struct SharedState {
    /// 睡眠时间是否已经都过完了
    completed: bool,

    /// Future 运行在任务上（总是属于一个任务），而那个任务有一个 Waker 就是这个 waker
    /// waker 就是 `TimerFuture` 所运行于的任务的 Waker（）
    /// 在设置 `completed = true` 之后，线程可以使用 waker 来告诉 `TimerFuture` 的任务可以唤醒了，并取得进展
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    // 查看 shared state，看下 timer 是否已经结束
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            /*
                设置 waker 以便当 timer 结束时线程可以唤醒当前任务，
                保证 Future 可以再次被 poll，并看到 `completed = true` 

                每次 Future 被 poll 时,都把 waker clone 一下，
                这时因为 TimerFuture 可在执行者的任务间移动，这会导致过期的 waker
                指向错误的任务，从而阻止了 TimerFuture 正确的唤醒

                Note：可以使用 `Waker::will_wake` 函数来检查这一点，为了简单这里我们就省略了
             */
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending            
        }
    }

}

impl TimerFuture {
    // 创建一个新的 TimerFuture，它将在提供的时限过后完成
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // 生成新线程
        // 这里 clone 一下，因为其是 Arc 类型所以只是增加了引用计数
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            // 线程休眠对应的时间
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();

            // 休眠时间已到了然后发出信号：计时器已停止并唤醒 Future 被 poll 的最后一个任务（如果存在的话）
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                // wake 方法被调用后，相关的任务（或 Future）就可以被唤醒，然后这个 Future 就会被再 poll 一下，
                // 看是否可以取得更多进展或完成
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}
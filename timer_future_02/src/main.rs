use futures::executor::block_on;
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
    thread,
    time::Duration,
};

// lib.rs 中的 TimerFuture
use timer_future_02::TimerFuture;


/// 任务执行者，它会从 channel 收到任务并运行它们
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` 产生新的 Futures 任务，并把任务放到 channel 中
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// 一个任务可以重新安排自己，以便被一个 `Executor` 来进行 poll
struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need to use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    /// 
    /// 正在进行中的 Future，它应该被推向完成
    /// `Mutex` 对应正确性来说不是必要的，因为我们同时只有一个线程在执行任务
    /// 尽管如此，Rust 不够聪明，它无法指定 future 只由一个线程来修改
    /// 所以我们需要使用 `Mutex` 来保证线程的安全
    /// 生成版本的执行者不需要这个，可以使用 `UnsafeCell` 来代替
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle to place the task itself back onto the task queue.
    /// 能把任务本身放回任务队列的处理器
    task_sender: SyncSender<Arc<Task>>,
}

/// 最开始会调用这个函数，返回一个执行者、一个任务生成器、和一个管道，
/// 执行者就是在管道的接收端
/// 任务生成器就是在管道的发送端，往通道里发送任务
fn new_executor_and_spawner() -> (Executor, Spawner) {
    // 在 channel 中允许同时排队的最大任务数
    // 这只是让 sync_channel 激活，并不会出现在真实的执行者中
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    println!("[{:?}] 生成 Executor 和 Spawner（含发送端、接收端）...", thread::current().id());
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        /// 将 future 包装成 任务
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        println!("[{:?}] 将 Future 组成 Task，放入 Channel ...", thread::current().id());
        /// 发送到通道
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        // 通过将该任务发送回任务 Channel 来实现 `wake`
        // 以便他将会被执行者再次进行 poll
        println!("[{:?}] call wake_by_ref ...", thread::current().id());

        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

impl Executor {
    fn run(&self) {
        println!("[{:?}] Executor running...", thread::current().id());
        // 从通道不断地接收任务，直到没有任务再继续往下走
        while let Ok(task) = self.ready_queue.recv() {
            println!("[{:?}] 接收到任务...", thread::current().id());
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            // 获得 future，如果它还没有完成（仍然是 Some），
            // 对它进行 poll，以尝试完成它
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                
                println!("[{:?}] 从任务中取得 Future...", thread::current().id());
                // Create a `LocalWaker` from the task itself
                // 从任务本身创建一个 `LocalWaker`
                
                let waker = waker_ref(&task); 
                println!("[{:?}] 获得 waker by ref ...", thread::current().id());

                let context = &mut Context::from_waker(&waker);
                println!("[{:?}] 获得 context 准备进行 poll() ...", thread::current().id());
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`

                // from it by calling the `Pin::as_mut` method.
                // `BoxFuture<T>` 是 `Pin<Box<dyn Future<Output = T> + Send + 'static>>` 的类型别名
                // 我们可以通过调用 `Pin::as_mut` 从它获得 `Pin<&mut dyn Future + Send + 'static>`
                if future.as_mut().poll(context).is_pending() {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    // 还没有对 Future 完成处理，所以把它放回它的任务
                    // 以便在未来再次运行
                    *future_slot = Some(future);
                    println!("[{:?}] Poll::Pending ====", thread::current().id());
                } else {
                    // 当返回 ready 后，这个通道就不会再有任务了，因为 main 函数中 spawner drop 操作
                    // 这时，这个 while 循环就停止了
                    println!("[{:?}] Poll::Ready....", thread::current().id());
                }
            }
        }
        println!("[{:?}] Executor run 结束", thread::current().id());
    }
}


// fn main() {
//     let future = TimerFuture::new(Duration::new(3, 0));
//     block_on(future);
// }

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // Spawn a task to print before and after waiting on a timer.
    // 生成一个任务，让其等待一个 timer 前后进行打印
    spawner.spawn(async {
        println!("[{:?}] howdy!", thread::current().id());
        // 等待 timer future 在 2s 后完成
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("[{:?}] spawner async done!", thread::current().id());
    });

    // 丢弃生成器以便我们的执行者知道它已经完成了
    drop(spawner);
    println!("[{:?}] Drop Spawner!", thread::current().id());


    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    // 运行执行者知道任务队列尾空为止
    // 这会打印 “howdy!”, 暂停，然后打印 “done!”.
    executor.run();
}

// #![allow(unused)]
// fn main() {
//     println!("Hello, world!");

//     async fn do_something() {

//     }
// }

use futures::executor::block_on;

// async fn hello_world() {
//     println!("hello, world!");
// }

// fn main() {
//     let future = hello_world(); // 未打印任何东西

//     // 将 future 传给执行者 block_on
//     block_on(future);  // future 才运行，并打印出 "hello, world!"
// }


struct Song {

}

async fn learn_song() -> Song {
    Song {}
}

async fn sing_song(song: Song) {}

async fn dance() {}

async fn learn_and_sing() {
    let song = learn_song().await;
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing(); // 返回 future
    let f2 = dance();          // 返回 future

    // join! 这个宏类似 await，它可以等待多个 future
    // 如果阻塞在了 f1 这个线程，那么 f2 线程就会接管当前线程，反之亦然
    // 如果 f1 和 f2 都阻塞了，那么就说这个函数阻塞了，那就需要交给其执行者，这里是 block_on
    futures::join!(f1, f2);
}

fn main() {
    // 下面 3 行是串行的，没有达到更好的性能要求
    // let song = block_on(learn_song());
    // block_on(sing_song(song));
    // block_on(dance());

    block_on(async_main());

}

//  Future 的一个简单实现
enum Poll<T> {
    Ready(T),
    Pending,
}

trait SimpleFuture {
    // Output 即未来要返回的值的类型
    type Output; 


    // poll 类似轮询，调用 poll 方法就会驱动 SimpleFuture 向着完成继续前进;
    // 参数 wake 是函数指针;
    // 返回值 Poll：
    // 1 如果是 Ready 就说明这个 Future 结束了，并且 Ready 的值的类型就是 Output 类型
    // 2 如果是 Pending 就说明这个 Future 还没有结束，未来至少还有 poll 一次，看看到时候进展
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}


// 例子一
pub struct SocketRead<'a> {
    socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // socket 有数据，读取数据到 buffer 并返回
            Poll::Ready(self.socket.read_buf())
        } else {
            /* 
                socket 还没有数据时，在未来有数据时，或这个 future 准备取得更多进展时候，
                带告诉它，就是通过 wake 这个函数告诉，当未来有数据时候，wake 就会被调用
                wake 被调用后就会再次调用 poll 这个方法来检查数据是否真的有了
                最后返回 Pending 变体
            */
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}


// 例子二
/*
    Join 中有两个 Future
    它的作用就是并发的让这两个 Future 来完成
*/
pub struct Join<FutureA, FutureB> {
    a: Option<FutureA>,
    b: Option<FutureB>,
}

impl<FutureA, FutureB> SimpleFuture for Join<FutureA,FutureB> 
where FutureA: SimpleFuture<Output = ()>, 
    FutureB:SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {

        // a 如果有值就取出，同时将 a 设置为 None
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(wake) {
                self.a.take();
            }
        }

        // b 如果有值就取出，同时将 b 设置为 None
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(wake) {
                self.b.take();
            }
        }

        // 如果 a 和 b 都是 None 就说明这两个 Future 都完成了
        // 否则就返回 Pending，Pending 就表示这里至少还有一个 Future 未完成 
        if self.a.is_none() && self.b.is_none() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
    
}


// 例子三
pub struct AndThenFut<FutureA, FutureB> {
    first: Option<FutureA>,
    second: FutureB,
}

impl<FutureA, FutureB> SimpleFuture for AndThenFut<FutureA, FutureB>
where FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()> {
    
    type Output = ();

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        // 如果 first 有值，就调用 first 的 poll 方法
        // 如果 返回 Ready 就说明第一个完成了，就把 first 设置为 None，然后再调用 second poll 进行返回
        // 如果 first 没有完成就返回 Pending
        if let Some(first) = &mut self.first {
            match first.poll(wake) {
                Poll::Ready(()) => self.first.take(),
                Poll::Pending => return Poll::Pending,
            };
        }
        self.second.poll(wake)
    }
}

// 真正的 Future Trait
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>,) -> Poll<Self::Output>;
}
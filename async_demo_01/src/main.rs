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


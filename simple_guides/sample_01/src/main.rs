// Demo1
// use std::thread::{self, sleep};
// use std::time::Duration;


/*
    #[tokio::main] 告诉编译器它使用 tokio 作为 async 运行时
*/

// #[tokio::main]
// async fn main() {
//     println!("Hello before reading file!");

//     // 生成异步运行任务，由 tokio 进行管理
//     let h1 = tokio::spawn(async {
//         /*
//             Note：由于 async 函数是惰性的，只有遇到 await 才会执行
//             // 这里必须调用 await 否则函数不会执行
//          */
//         let _file1_contents = read_from_file1().await;
//     });

//     let h2 = tokio::spawn(async {
//         let _file2_contents = read_from_file2().await;
//     });

//     let _ = tokio::join!(h1, h2);
// }

/*
    async fn 变为可被 tokio 安排的异步运行时任务
    async 函数是惰性的，只有遇到 await 才会执行
*/
// async fn read_from_file1() -> String {
//     sleep(Duration::new(4, 0));
//     println!("{:?}", "Processing file 1");
//     String::from("Hello, there from file 1")
// }

// async fn read_from_file2() -> String {
//     sleep(Duration::new(2, 0));
//     println!("{:?}", "Processing file 2");
//     String::from("Hi, there from file 2")
// }


// Demo2 
// use std::thread::{self, sleep};
// use std::time::Duration;


// fn main() {
//     println!("Hello before reading file!");
//     let handle1 = thread::spawn(|| {
//         let file1_contents = read_from_file1();
//         println!("{:?}", file1_contents);
//     });

//     let handle2 = thread::spawn(|| {
//         let file2_contents = read_from_file2();
//         println!("{:?}", file2_contents);
//     });

//     handle1.join().unwrap();
//     handle2.join().unwrap();
// }

// fn read_from_file1() -> String {
//     sleep(Duration::new(4, 0));
//     String::from("Hello, there from file 1")
// }

// fn read_from_file2() -> String {
//     sleep(Duration::new(2, 0));
//     String::from("Hi, there from file 2")
// }

// use std::future::Future;

// fn read_from_file1() -> impl Future<Output = String> {
//     async {
//         sleep(Duration::new(4, 0));
//         println!("{:?}", "Processing file 1");
//         String::from("Hello, there from file 1")
//     }
// }

// Demo3

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread::sleep;
use std::time::Duration;

struct ReadFileFuture {}

impl Future for ReadFileFuture {
    type Output = String;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("Tokio! Stop polling me");
        /*
            通知 tokio 运行时，异步任务已经准备好了，
            可以被 poll 了

            这里因为下边返回的还是 Poll::Pending，
            所以这个方法会被不断地执行
        */
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

async fn read_from_file2() -> String {
    sleep(Duration::new(2, 0));
    println!("{:?}", "Processing file 2");
    String::from("Hi, there from file 2")
}

#[tokio::main]
async fn main() {
    println!("Hello before reading file!");

    // 生成异步任务
    let h1 = tokio::spawn(async {
        let future1 = ReadFileFuture {};
        future1.await;
    });

    let h2 = tokio::spawn(async {
        let file2_contents = read_from_file2().await;
        println!("{:?}", file2_contents);
    });

    let _ = tokio::join!(h1, h2);
}


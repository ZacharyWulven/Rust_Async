// 7.1 async ? 临时解决方案
// #![allow(unused)]
// fn main() {
//     struct MyError;
    
//     async fn foo() -> Result<(), MyError> {
//         Ok(())
//     }

//     async fn bar() -> Result<(), MyError> {
//         Ok(())
//     }

//     let fut = async {
//         foo().await?;
//         bar().await?;
//         // Ok(()) // Error, 无法推断 Result 上边的 E 的类型
//         Ok::<(), MyError>(()) // 临时解决方案
//     };
// }


// 7.2 Send traint Approximation
// use std::rc::Rc;
// #[derive(Default)]
// struct NotSend(Rc<()>); // 不是 Send 的，里边有个 Rc
// async fn bar() {}
// async fn foo() {
//     //  因为 NotSend::default() 这样调用，只在 foo 里出现了一下，这样是没有问题的，编译器不会报错
//     // NotSend::default();
    
//     // 但如果我们获取 NotSend 的返回值，下边 main 函数就会报错
//     // Error: future returned by `foo` is not `Send
//     // let x = NotSend::default();

//     // 临时解决方案：引入块作用域
//     {
//         let x = NotSend::default();

//     }

//     bar().await;
// }

// fn require_send(_: impl Send) {}

// fn main() {
//     require_send(foo());
// }


// 7.3 Recursion

// This function:
// async fn foo() {
//     step_one().await;
//     step_two().await;
// }
// generates a type like this:
// 编译后对 foo 函数解析，产生的类型
// enum Foo {
//     First(StepOne),
//     Second(StepTwo),
// }



// So this function:
// async fn recursive() {
//     recursive().await;
//     recursive().await;
// }

/*
    编译后对 recursive 函数解析，产生的类型
    这样就不行了，因为这样会产生一个无限大小的类型，编译器无法知道其大小，这时就会报错
    Error: a recursive `async fn` must be rewritten to return a boxed `dyn Future`
*/
// enum Recursive {
//     First(Recursive),
//     Second(Recursive),
// }


use futures::future::{BoxFuture, FutureExt};

fn recursive() -> BoxFuture<'static, ()> {
    async move {
        recursive().await;
        recursive().await;
    }.boxed()
}

fn main() {
    
}




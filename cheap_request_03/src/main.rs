
use std::future::Future;

use async_std::io::prelude::*;
use async_std::net;
use async_std::task;

/// 异步函数以 async 开头
/// 虽然返回值是 std::io::Result<String>，但无需调整返回值类型，Rust 自动把它当成相应的 Future 类型
/// 返回的 Future 是包含所需相关信息的：包括参数、本地变量空间
/// 
/// Future 的具体类型是由编译器基于函数体和参数自动生成的
/// 1. 改类型没有名称
/// 2. 它实现了 Future<Output=R>，这个函数 R 就是 Result<String>
/// 
/// 
/// 第一次对 cheapo_request 进行 poll 时：
/// 从函数体顶部开始执行，直到第一个 await（针对 TcpStream::connect 返回 Future），
/// 这个 await 就会对 TcpStream::connect 的 Future 进行 poll
/// 1. 如果没有完成就返回 Pending
/// 2. 只要没有完成， cheapo_request 函数就没法继续
/// 3. main 函数中对 cheapo_request 也无法继续 poll
/// 4. 直到 TcpStream::connect 返回 Ready 后才能继续往后执行
/// 
/// 
/// await 能干什么？：
/// 1. 获得 Future 的所有权，并对其进行 poll
/// 2. 对 Future 进行 poll 时，如果 Future 返回 Ready，其最终值就是 await 表达式，这时就继续执行后续代码，
/// 否则就返回 Pending 给调用者
/// 
/// 
/// Note：
/// 下一次 main 函数中对 cheapo_request 的 Future 进行 poll 时（使用执行器 block_on），
/// 并不是从函数体顶部开始执行，而是从上一次暂停的位置开始，直到它变成 Ready，才会继续在函数体往下走
/// 例如：TcpStream::connect 上一次返回 Pending，那么代码就停在这里了，下次会从这里再开始看能否
/// 取得更多进展
/// 小结：可以理解为 await Pending 时候，相当于 suspended 了，下次 poll 从这里恢复直到 Ready，再往下走
/// 
/// 
/// 随着 cheapo_request 的 Future 不断被 poll，其执行就是从一个 await 到下一个 await，而且只有子 Future
/// 的 await 变成 Ready 之后才能继续往下走。
/// cheapo_request 的 Future 会追踪：
/// 1. 下一次 poll 应该恢复继续的那个点
/// 2. 以及所需的本地状态（变量、参数、临时变量等）
/// 
/// 
/// 这种途中能暂停执行，然后恢复执行的能力是 async 所独有的，由于 await 表达式依赖于“可恢复执行”这个特性，
/// 所以 await 只能用在 async 中
/// 而暂停执行时线程在做什么？它不是在干等着，而是在做其他的工作。
///
/// 
async fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String> {
    /*
        .await 的是异步的
        .await 会等待，直到 Future 变成 ready，ready 后 await 最终会解析出 Future 的值

        Note：当调用 async 函数时，在其函数体执行前，它就会立即返回，即执行到 port)) 函数就返回了

        这里就是获取 TcpStream::connect 返回 Future 的所有权，并对这个 Future 进行 poll，
        但对 Future 进行 poll，await 不是唯一的方式，其他的执行器也可以

     */
    let mut socket = net::TcpStream::connect((host, port)).await?;
    let request = format!("Get {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);

    socket.write_all(request.as_bytes()).await?;
    socket.shutdown(net::Shutdown::Write)?;

    let mut response = String::new();
    socket.read_to_string(&mut response).await?;
    Ok(response)
}

fn main() -> std::io::Result<()> {
    // 在非异步函数中调用异步函数 block_on 是一种方式, 它是一个执行器
    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}


// This function:
async fn foo(x: &u8) -> u8 { *x }

// Is equivalent to this function
fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    async move { *x }
}
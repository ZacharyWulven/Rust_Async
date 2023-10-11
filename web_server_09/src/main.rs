use async_std::{ net::{TcpListener, TcpStream}, prelude::*, task::{self, spawn} };
use futures::stream::StreamExt;
use std::fs;
use std::time::Duration;

// use for tests 
use async_std::io::{Read, Write};


#[async_std::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    /*
        标准库中的 TcpListener 的 incoming() 是阻塞的，
        这里 async_std 的 TcpListener 的 incoming() 就不是非阻塞的，它返回一个流

        for_each_concurrent 用于并发处理 stream 中的元素
        参数 1：是并发处理最大极限，这里传 None
        参数 2：是个闭包
     */
    listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |tcpstream| async move {
            let tcpstream = tcpstream.unwrap();
            //handle_connection(tcpstream).await;
            spawn(handle_connection(tcpstream));
        })
        .await;
}
// TcpStream 来自 async_std，之前 TcpStream 是标准库的
// async fn handle_connection(mut stream: TcpStream) {
async fn handle_connection(mut stream: impl Read + Write + Unpin) {

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap(); // async version

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{status_line}{contents}");
    stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::io::Error;
    use futures::task::{Context, Poll};
    use std::cmp::min;
    use std::pin::Pin;

    struct  MockTcpStream {
        read_data: Vec<u8>,
        write_data: Vec<u8>,
    }

    impl Read for MockTcpStream {
        fn poll_read(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &mut [u8],
                ) -> Poll<std::io::Result<usize>> {
                    // 读取 read_data 长度与 buffer 长度中比较小的值
                    let size: usize = min(self.read_data.len(), buf.len());
                    // 将 read_data 数据 copy 到 buffer 中
                    buf[..size].copy_from_slice(&self.read_data[..size]);
                    // 返回 Ready 表示完成
                    Poll::Ready(Ok(size))
        }
    }

    impl Write for MockTcpStream {
        // 把数据写入 TcpStream，
        fn poll_write(mut
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &[u8],
                ) -> Poll<std::io::Result<usize>> {
                    self.write_data = Vec::from(buf);
                    Poll::Ready(Ok(buf.len()))   
        }

        // 针对 MockTcpStream 而言，poll_flush 和 poll_close 就没啥用，返回 Poll::Ready 就可以了
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    // 实现 Unpin
    use std::marker::Unpin;
    impl Unpin for MockTcpStream {


    }

    use std::fs;

    #[async_std::test] // 标记为异步测试函数
    async fn test_handle_connection() {
        let input_bytes = b"GET / HTTP/1.1\r\n";
        let mut contents = vec![0u8; 1024];
        contents[..input_bytes.len()].clone_from_slice(input_bytes);

        // mock MockTcpStream
        let mut stream = MockTcpStream {
            read_data: contents,
            write_data: Vec::new(),
        };

        handle_connection(&mut stream).await;
        // let mut buf: [u8; 1024] = [0u8; 1024];
        // stream.read(&mut buf).await.unwrap();

        let expected_contents = fs::read_to_string("hello.html").unwrap();
        let expected_response = format!("HTTP/1.1 200 OK\r\n\r\n{}", expected_contents);
        // 看 stream.write_data 是不是以 expected_response 开头
        assert!(stream.write_data.starts_with(expected_response.as_bytes()));
    }



}
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::TcpListener,
    time::sleep,
};
use std::sync::Arc;
use tokio::sync::Semaphore;
use iec104::Settings;


// 异步处理接收到的数据
async fn process_data(data: Vec<u8>) {
    // 这里是示例处理函数，打印接收到的字节
    println!("Received {} bytes: {:?}", data.len(), data);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let settings = Settings::new()?;
    
    // 创建 TCP 监听器
    let listener = TcpListener::bind(settings.server_addr()).await?;
    println!("Server listening on {}", settings.server_addr());

    // 使用信号量限制并发连接数
    let semaphore = Arc::new(Semaphore::new(settings.server.max_connections));
    println!("Connection limit set to {}", settings.server.max_connections);

    loop {
        // 获取信号量许可
        let permit = semaphore.clone().acquire_owned().await?;
        
        // 使用 tokio::select! 来处理超时
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((socket, addr)) => {
                        println!("New connection from: {}", addr);
                        
                        // 为每个连接创建一个新的异步任务
                        let buffer_size = settings.server.buffer_size;
                        let timeout = settings.connection_timeout();
                        
                        tokio::spawn(async move {
                            let _permit = permit; // 当任务结束时自动释放许可
                            let mut reader = BufReader::new(socket);
                            let mut buffer = vec![0; buffer_size];
                            
                            loop {
                                tokio::select! {
                                    read_result = reader.read(&mut buffer) => {
                                        match read_result {
                                            Ok(n) if n == 0 => {
                                                println!("Connection closed by client: {}", addr);
                                                break;
                                            }
                                            Ok(n) => {
                                                // 创建包含实际数据的切片
                                                let data = buffer[..n].to_vec();
                                                // 异步处理数据
                                                process_data(data).await;
                                            }
                                            Err(e) => {
                                                eprintln!("Error reading from connection {}: {}", addr, e);
                                                break;
                                            }
                                        }
                                    }
                                    _ = sleep(timeout) => {
                                        println!("Connection timeout for: {}", addr);
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        sleep(settings.retry_delay()).await; // 出错时短暂延迟
                    }
                }
            }
        }
    }
}

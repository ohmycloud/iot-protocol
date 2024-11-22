use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};
use std::{sync::Arc, io::Result as IoResult};
use tokio::sync::Semaphore;
use bytes::{BytesMut, Buf, Bytes};
use iec104::Settings;

async fn process_data(data: Bytes) {
    println!("Received {} bytes: {:?}", data.len(), data);
}

// 处理单个客户端连接
async fn handle_client(mut socket: TcpStream, addr: std::net::SocketAddr, settings: Arc<Settings>) -> IoResult<()> {
    let mut buf = BytesMut::with_capacity(1024);
    
    loop {
        tokio::select! {
            // 读取数据
            read_result = socket.read_buf(&mut buf) => {
                match read_result? {
                    0 => {
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    n => {
                        if n >= 2 {  // 至少包含启动字符和长度字段
                            // 检查是否收到完整的帧
                            while buf.len() >= 2 {
                                if buf[0] != 0x68 {  // 检查启动字符
                                    buf.advance(1);  // 跳过无效字节
                                    continue;
                                }
                                
                                let length = buf[1] as usize + 2;  // APDU长度 + 启动字符和长度字段
                                if buf.len() >= length {
                                    // 提取完整的帧
                                    let frame = buf.split_to(length).freeze();
                                    process_data(frame).await;
                                } else {
                                    break;  // 等待更多数据
                                }
                            }
                        }
                    }
                }
            }
            // 超时检查
            _ = sleep(settings.connection_timeout()) => {
                println!("Connection timeout for client {}", addr);
                break;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let settings = Arc::new(Settings::new()?);
    
    // 创建 TCP 监听器
    let listener = TcpListener::bind(settings.server_addr()).await?;
    println!("Server listening on {}", settings.server_addr());

    // 使用信号量限制并发连接数
    let semaphore = Arc::new(Semaphore::new(settings.server.max_connections));
    println!("Connection limit set to {}", settings.server.max_connections);

    loop {
        // 获取信号量许可
        let permit = semaphore.clone().acquire_owned().await?;
        
        // 等待客户端连接
        let (socket, addr) = listener.accept().await?;
        println!("Client connected from {}", addr);

        // 为每个连接创建一个新的任务
        let settings = settings.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, addr, settings).await {
                eprintln!("Error handling client {}: {}", addr, e);
            }
            // 释放信号量许可
            drop(permit);
        });
    }
}

use bytes::{BytesMut, BufMut, Bytes};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[tokio::test]
async fn test_frame_transmission() -> Result<(), Box<dyn std::error::Error>> {
    // 启动测试服务器
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // 在新任务中运行服务器
    let server_handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = BytesMut::with_capacity(1024);
        
        // 读取一个完整的帧
        socket.read_buf(&mut buf).await.unwrap();
        assert_eq!(buf[0], 0x68); // 检查启动字符
        assert_eq!(buf[1], 0x04); // 检查长度字段
        assert_eq!(buf[2], 0x07); // 检查 STARTDT act
    });

    // 创建客户端连接
    let mut client = TcpStream::connect(addr).await?;

    // 创建并发送 STARTDT act 帧
    let mut frame = BytesMut::with_capacity(6);
    frame.put_u8(0x68);   // 启动字符
    frame.put_u8(0x04);   // APDU 长度
    frame.put_u8(0x07);   // STARTDT act
    frame.put_u8(0x00);
    frame.put_u8(0x00);
    frame.put_u8(0x00);
    
    client.write_all(&frame).await?;

    // 等待服务器完成处理
    server_handle.await?;

    Ok(())
}

#[tokio::test]
async fn test_i_frame_transmission() -> Result<(), Box<dyn std::error::Error>> {
    // 启动测试服务器
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // 在新任务中运行服务器
    let server_handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = BytesMut::with_capacity(1024);
        
        // 读取一个完整的帧
        socket.read_buf(&mut buf).await.unwrap();
        assert_eq!(buf[0], 0x68); // 检查启动字符
        assert_eq!(buf[1], 0x0E); // 检查长度字段 (14 bytes)
        assert_eq!(buf[2], 0x00); // 检查 I-Frame 控制字段
    });

    // 创建客户端连接
    let mut client = TcpStream::connect(addr).await?;

    // 创建并发送测试 I-Frame
    let mut frame = BytesMut::with_capacity(16);
    frame.put_u8(0x68); // Start
    frame.put_u8(0x0E); // Length of APDU (14 bytes)
    frame.put_u8(0x00); // Control field 1 - I-Frame
    frame.put_u8(0x00); // Control field 2
    frame.put_u8(0x00); // Control field 3
    frame.put_u8(0x00); // Control field 4
    frame.put_u8(0x64); // TypeID: C_SE_NC_1 (100)
    frame.put_u8(0x01); // VSQ
    frame.put_u8(0x06); // COT
    frame.put_u8(0x00); // Common Address (low)
    frame.put_u8(0x00); // Common Address (high)
    frame.put_u8(0x01); // Info Object Address (low)
    frame.put_u8(0x00); // Info Object Address (medium)
    frame.put_u8(0x00); // Info Object Address (high)
    frame.put_f32(3.14); // Normalized value

    client.write_all(&frame).await?;

    // 等待服务器完成处理
    server_handle.await?;

    Ok(())
}

use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{sleep, Duration},
};

// IEC104 APCI 类型
const START_BYTE: u8 = 0x68;

// APCI 类型
#[derive(Debug)]
enum ApciType {
    IFrame = 0x00,     // 信息传输格式
    SFrame = 0x01,     // 监视格式
    UFrame = 0x03,     // 无编号控制格式
}

// 创建 U-Frame 启动帧
fn create_startdt_act() -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(START_BYTE);   // 启动字符
    frame.push(4);      // APDU 长度
    frame.push(0x07);   // U-Frame 控制域 1 (STARTDT act)
    frame.push(0x00);   // U-Frame 控制域 2
    frame.push(0x00);   // U-Frame 控制域 3
    frame.push(0x00);   // U-Frame 控制域 4
    frame
}

// 创建测试 I-Frame
fn create_test_i_frame() -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(START_BYTE);   // 启动字符
    frame.push(14);     // APDU 长度
    
    // 控制域 (I-Format)
    frame.push(0x00);   // 发送序号低字节
    frame.push(0x00);   // 发送序号高字节
    frame.push(0x00);   // 接收序号低字节
    frame.push(0x00);   // 接收序号高字节

    // ASDU
    frame.push(0x2D);   // 类型标识符 (45: 单命令)
    frame.push(0x01);   // 可变结构限定词
    frame.push(0x06);   // 传送原因
    frame.push(0x00);   // 传送原因高字节
    frame.push(0x01);   // 公共地址低字节
    frame.push(0x00);   // 公共地址高字节
    
    // 信息对象
    frame.push(0x01);   // 信息对象地址低字节
    frame.push(0x00);   // 信息对象地址高字节
    frame.push(0x01);   // 单命令值

    frame
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到服务器
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    // 发送 STARTDT act
    let startdt = create_startdt_act();
    stream.write_all(&startdt).await?;
    println!("Sent STARTDT activation");
    
    // 等待一下
    sleep(Duration::from_secs(1)).await;

    // 发送测试数据帧
    loop {
        let i_frame = create_test_i_frame();
        stream.write_all(&i_frame).await?;
        println!("Sent test I-Frame");
        
        // 每秒发送一次
        sleep(Duration::from_secs(1)).await;
    }
}

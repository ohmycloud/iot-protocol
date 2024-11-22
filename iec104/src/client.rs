use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{sleep, Duration},
};
use bytes::{BytesMut, BufMut, Bytes};

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
fn create_startdt_act() -> Bytes {
    let mut buf = BytesMut::with_capacity(6);
    buf.put_u8(START_BYTE);  // 启动字符
    buf.put_u8(4);        // APDU 长度
    buf.put_u8(0x07);     // U-Frame 控制域 1 (STARTDT act)
    buf.put_u8(0x00);     // U-Frame 控制域 2
    buf.put_u8(0x00);     // U-Frame 控制域 3
    buf.put_u8(0x00);     // U-Frame 控制域 4
    buf.freeze()
}

// 创建测试 I-Frame
fn create_test_i_frame() -> Bytes {
    let mut buf = BytesMut::with_capacity(16);
    // APCI
    buf.put_u8(0x68);  // Start
    buf.put_u8(0x0E);  // Length of APDU (14 bytes)
    buf.put_u8(0x00);  // Control field 1 - I-Frame
    buf.put_u8(0x00);  // Control field 2
    buf.put_u8(0x00);  // Control field 3
    buf.put_u8(0x00);  // Control field 4

    // ASDU
    buf.put_u8(0x64);  // TypeID: C_SE_NC_1 (100) - 设定命令，规一化值
    buf.put_u8(0x01);  // VSQ: 单个信息对象
    buf.put_u8(0x06);  // COT: 激活
    buf.put_u8(0x00);  // Common Address of ASDU (低字节)
    buf.put_u8(0x00);  // Common Address of ASDU (高字节)
    buf.put_u8(0x01);  // Information Object Address (低字节)
    buf.put_u8(0x00);  // Information Object Address (中字节)
    buf.put_u8(0x00);  // Information Object Address (高字节)
    buf.put_f32(3.14); // 规一化值

    buf.freeze()
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

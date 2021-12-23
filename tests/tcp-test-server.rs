#[allow(unused_imports)]
use async_std::net::{TcpStream, TcpListener};
use futures::{StreamExt};
use async_std::{
    task
};
use log4rs;
pub use log::{
    info,
    debug,
    error,
    warn,
};
// use futures-util

#[async_std::test]
async fn accept_loop() -> Result<(), std::io::Error> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    let addr = "127.0.0.1:10000";
    debug!("准备监听地址：{}", addr);
    let listener = TcpListener::bind(addr).await?;
    debug!("监听地址：{}成功", addr);
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        debug!("Accepting From: {}", stream.peer_addr()?);
        let _handle = task::spawn(connect_loop(stream));
    };
    Ok(())
}

#[allow(unreachable_code)]
pub async fn connect_loop(stream:TcpStream) -> Result<(), std::io::Error>{

    let (reader, writer) = &mut (&stream, &stream);
    // let buf = &mut [0u8; 100];
    // let other = &[8u8, 11, 23,12,34];
    // let len = buf.len();
    // let duration = Duration::from_secs(5);
    loop {
        debug!("开始转发数据");
        async_std::io::copy(reader, writer).await.unwrap();
        break;
        // let data = stream.read(buf).await?;
        // // buf_reader.consume_unpin(data.len());
        // debug!("fill_buf()={:?}", data);
        // debug!("fill_buf()={:?}", buf);
        // stream.write_all(&buf[0..20]).await;
        // task::sleep(duration).await;
        // stream.write_all(other).await;
    }
    debug!("end to receive client");
    Ok(())
}

use async_std::net::{TcpStream};
use futures::{select, AsyncWriteExt};
use futures::FutureExt;
use async_std::{
    task
};
use async_std::io::ReadExt;
use log4rs;
pub use log::{
    info,
    debug,
    error,
    warn,
};
use std::time::Duration;

#[async_std::test]
async fn connect_exchange() -> Result<(), std::io::Error> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    let addr = "127.0.0.1:10000";
    debug!("开始连接目标{}...", addr);
    if let Ok(stream) = TcpStream::connect(addr).await {
        debug!("目标{}连接成功...", addr);
        let (mut reader, mut writer) = (&stream, &stream);
        // let buffer = &mut Vec::<u8>::with_capacity(1024);
        // let mut buf_reader = BufReader::new(reader);

        let n = writer.write(&[8u8, 1u8, 2u8,23u8]).await?;
        let acb = &mut[0u8; 30];
        debug!("写数据成功: {}", n);
        let mut vals = vec![11u8, 29u8];
        let duration = Duration::from_secs(3);
        // let tmp = &acb[0..5];
        loop {
            task::sleep(duration).await;
            writer.write(vals.as_ref()).await.unwrap();
            select! {
                val0 = reader.read(acb).fuse()=> match val0 {
                    Ok(abc) => {
                        debug!("读数据(长度{})成功: {:?}", abc, &acb[0..abc]);
                        vals.append(&mut (&acb[0..abc]).to_vec());
                    },
                    Err(e) => {
                        debug!("err={:?}", e);
                        break;
                    }
                }
            }
        }
        debug!("****************");
        Ok(())
    } else  {
        debug!("无法连接目标");
        Ok(())
    }
}

use iot_gateway::{get_ip, Result};
mod common;
use common::*;

// mulIp = "224.0.211.211"
// mulListenPort = 6100
// mulGatewayPort = 6000
#[async_std::test]
async fn nodemcu() -> Result<()> {
    init_log_config();
    let mut ip = get_ip()?;
    let local = ip.to_string();
    ip.push_str(":6100");
    use std::net::UdpSocket;
    let udp = UdpSocket::bind(ip)?;
    udp.join_multicast_v4(&"224.0.211.211".parse()?, &local.parse()?)?;
    udp.send_to(
        "{\"action\":\"heart\",\"uuid\":\"45abc6\",\"from\":\"123\",\"to\":\"netgate\", \"type\":\"req\"}".as_bytes(),
        "224.0.211.211:6000",
    );
    let mut data = [0u8; 1024];
    let mut i = 5;
    loop {
        match udp.recv(&mut data) {
            Ok(size) => {
                let msg = String::from_utf8_lossy(&data[0..size]);
                debug!("{:?}", msg);
                udp.send_to(
                    "{\"action\":\"heart\",\"uuid\":\"45abc6\",\"from\":\"456\",\"to\":\"netgate\", \"type\":\"req\"}"
                        .as_bytes(),
                    "224.0.211.211:6000",
                );
            }
            Err(e) => {}
        }
        i -= 1;
        if i < 0 {
            break;
        }
    }
    Ok(())
}

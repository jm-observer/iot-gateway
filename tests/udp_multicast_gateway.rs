use async_std::net::ToSocketAddrs;
use iot_gateway::{get_ip, Result};
mod common;
use common::*;

// mulIp = "224.0.211.211"
// mulListenPort = 6100
// mulGatewayPort = 6000
#[test]
fn gateway() -> Result<()> {
    init_log_config();
    let mut ip = get_ip()?;
    debug!("ip={}", ip);
    let local = ip.to_string();
    ip.push_str(":6000");
    //239.255.255.250:3702
    use std::net::UdpSocket;
    let udp = UdpSocket::bind(ip)?;
    udp.join_multicast_v4(&"224.0.211.211".parse()?, &local.parse()?)?;
    let mut data = [0u8; 1024];
    loop {
        match udp.recv(&mut data) {
            Ok(size) => {
                let msg = String::from_utf8_lossy(&data[0..size]);
                debug!("{:?}", msg);
                udp.send_to("123".as_bytes(), "224.0.211.211:6100");
            }
            Err(e) => {}
        }
    }
    Ok(())
}

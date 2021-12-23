use async_std::net::UdpSocket;

#[async_std::main]
async fn main() {
    let local_ip = "192.168.50.9";
    let local_ip_port = "192.168.50.9:6100";
    let client = UdpSocket::bind(local_ip_port).await.unwrap();

    if let Err(e) =
        client.join_multicast_v4("224.0.211.211".parse().unwrap(), local_ip.parse().unwrap())
    {
        println!("{:?}", e);
        return;
    }

    // client.connect("224.0.0.11:6000").await.unwrap();
    client
        .send_to(&[0u8, 0, 0], "224.0.211.211:6000")
        .await
        .unwrap();
    // client.send(&[0u8, 0, 0]).await.unwrap();
    let data = &mut [0u8; 1024];
    println!("11111111111111{:?}", client.recv(data).await.unwrap());
    println!("11111111111111{:?}", client.recv(data).await.unwrap());
    println!("11111111111111{:?}", client.recv(data).await.unwrap());
    println!("11111111111111{:?}", client.recv(data).await.unwrap());
}

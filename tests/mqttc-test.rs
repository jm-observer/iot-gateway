mod common;

use common::*;

use async_std::task;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::str;
use std::time::Duration;

#[async_std::test]
async fn send_packet() {
    //_send("ssh", get_payload("ssh_start")).await;
    // _send("ssh", get_payload("ssh_end")).await;
    _send("heartCheck", get_payload("heartCheck")).await;
}

fn get_payload(event: &str) -> String {
    match event {
        "ssh_start" => "{\"ip\":\"148.70.132.77\",\"ah\":\"192.168.50.135\",\"ap\":22,\"port\":20004,\"u\":\"2101281930273003\",\"order\":\"start\"}".to_string(),
        "ssh_end" => "{\"ip\":\"148.70.132.77\",\"ah\":\"192.168.50.135\",\"ap\":22,\"port\":20004,\"u\":\"2101281930273003\",\"order\":\"end\"}".to_string(),
        "heartCheck" => "{\"u\":\"2101281930273111103\"}".to_string(),

        _ => "".to_string()
    }
}
#[async_std::test]
async fn receiver() {
    let (client, eventloop) = _init_async_mqtt_client("rustc-test-server").await;
    client
        .subscribe("/sbiot/#", QoS::ExactlyOnce)
        .await
        .unwrap();
    client
        .subscribe("/28D24499AB4E_support/#", QoS::ExactlyOnce)
        .await
        .unwrap();
    client
        .subscribe("/4CEDFB6D231A_support/#", QoS::ExactlyOnce)
        .await
        .unwrap();
    client
        .subscribe("/rustc-test-client/#", QoS::ExactlyOnce)
        .await
        .unwrap();
    _receiver(eventloop, false).await;
}

async fn _send(event: &str, payload: String) {
    let qos = QoS::AtLeastOnce;
    let (client, eventloop) = _init_async_mqtt_client("rustc-test-client").await;
    if let Err(e) = client
        .publish(
            format!(
                "/28D24499AB4E_support/req/server/rustc-test-client/{}",
                event
            ),
            qos,
            false,
            payload.as_str(),
        )
        .await
    {
        error!("{}", format!("{:?}", e));
    }
    if let Err(e) = client
        .publish(
            format!(
                "/4CEDFB6D231A_support/req/server/rustc-test-client/{}",
                event
            ),
            qos,
            false,
            payload.as_str(),
        )
        .await
    {
        error!("{}", format!("{:?}", e));
    }
    _receiver(eventloop, true).await;
}

async fn _receiver(mut connection: EventLoop, is_sender: bool) {
    let tasker = task::spawn(async move {
        let mut rec_time = 0;
        loop {
            match connection.poll().await {
                Ok(Event::Outgoing(ok)) => {
                    debug!("{:?}", ok);
                    rec_time = rec_time + 1;
                    if is_sender && rec_time >= 2 {
                        break;
                    }
                }
                Ok(Event::Incoming(Packet::Publish(packet))) => {
                    let msg = &packet.payload;
                    let payload = str::from_utf8(msg).unwrap();
                    debug!(
                        "receiver mqtt消息：topic={} msg={:?}",
                        packet.topic, payload
                    );
                }
                Err(e) => {
                    debug!("{:?}", e);
                    break;
                }
                _ => {}
            }
        }
    });
    task::block_on(tasker);
    task::sleep(Duration::from_secs(4)).await;
}

async fn _init_async_mqtt_client(mqtt_id: &str) -> (AsyncClient, EventLoop) {
    // log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    init_log_config();
    let mut mqttoptions = MqttOptions::new(mqtt_id, "mqtt.duduwuli.cn", 1883);
    mqttoptions.set_keep_alive(120);
    mqttoptions.set_credentials("netgate", "jio83hc739duyeh2");
    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
    return (client, eventloop);
}

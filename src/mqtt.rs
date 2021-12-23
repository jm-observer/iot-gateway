/// mqtt客户端实现
use crate::mqtt_receive;
use crate::*;

use futures::FutureExt;
use rumqttc::v4::{ConnAck, ConnectReturnCode, Packet};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS};
use std::collections::HashMap;

pub async fn mqtt(global_param: Arc<Global>) -> Result<()> {
    let rec_from_other = global_param.rec_from_other.clone();
    let (client, mut eventloop) = _init_async_mqtt_client(global_param.clone()).await?;
    let subscribe = format!("/{}/#", &global_param.mqtt_id);
    let tasker = task::spawn(async move {
        let mqtt_msg_map: Arc<Mutex<HashMap<String, MqttPacket>>> =
            Arc::new(Mutex::new(HashMap::new()));
        loop {
            select! {
                packet = eventloop.poll().fuse() => match packet {
                    Ok(Event::Incoming(Packet::Publish(packet))) => {
                        task::spawn(
                            mqtt_receive(packet, global_param.clone(), mqtt_msg_map.clone())
                        );
                    },
                    Err(e) => {
                        //Io(Os { code: 11001, kind: Other, message: "不知道这样的主机。" })
                        error!("Received = {:?}", e);
                        task::sleep(Duration::from_secs(10u64)).await;
                    },
                    Ok(Event::Outgoing(_outgoing)) => {
                        // debug!("{:?}", _outgoing);
                    },
                    Ok(Event::Incoming(Packet::ConnAck(ca))) => {
                        let ConnAck{session_present: _, code} = ca;
                        match code {
                            ConnectReturnCode::Success => {
                                if let Err(e) = client.subscribe(&subscribe, QoS::ExactlyOnce).await {
                                    error!("mqtt订阅失败：{:?}", e);
                                } else {
                                    info!("mqtt订阅成功！");
                                    if let Ok(msg) = global_param.get_heart_check_msg() {
                                        send_req_to_mqtt(global_param.clone(), "heartCheck", msg, None).await;
                                    }
                                }
                            },
                            e => {
                                error!("mqtt连接失败：{:?}", e);
                            }
                        }

                    },
                    _o => {
                        // debug!("{:?}", _o);
                    }
                },
                packet = rec_from_other.recv().fuse() => match packet {
                    Ok(packet) => {
                        let topic = packet.get_topic();
                        // let topic_tmp = topic.clone();
                        let payload = packet.get_payload();
                        debug!("接收需要发送的mqtt数据：{}--{}", topic, payload);
                        if let Err(err) = client.publish(topic, QoS::ExactlyOnce, false, payload).await {
                            warn!("消息：{:?}发送失败：{:?}", packet.get_u(), err);
                        }
                        // debug!("mqtt数据发送成功：{}--{}", topic_tmp, payload);
                        if packet.need_follow_up {
                            //TODO 需要后续跟踪清理
                            let mut tmp_map = mqtt_msg_map.lock().await;
                            tmp_map.insert(packet.get_u(), packet);
                        }
                    },
                    Err(e) => {
                        error!("接收需要发送的mqtt数据异常：{:?}", e);
                    }
                }
            }
        }
    });
    task::block_on(tasker);
    Ok(())
}

async fn _init_async_mqtt_client(global: Arc<Global>) -> Result<(AsyncClient, EventLoop)> {
    let config = global.toml_config.read().await;
    let mqtt_password = config.get_two_level_string("mqtt", "password")?;
    let mqtt_user = config.get_two_level_string("mqtt", "user")?;
    let mqtt_server = config.get_two_level_string("mqtt", "server")?;
    let mqtt_port = config.get_two_level_int("mqtt", "port")?;
    let mqtt_port: u16 = mqtt_port as u16;
    let mut mqttoptions = MqttOptions::new(global.mqtt_id.clone(), mqtt_server, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(120));
    mqttoptions.set_credentials(mqtt_user, mqtt_password);
    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
    return Ok((client, eventloop));
}

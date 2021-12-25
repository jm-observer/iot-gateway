mod config;

use crate::mmqtt::config::MMqttConfig;
use crate::pub_use::*;
use crate::*;
use futures::{select, FutureExt};
use rumqttc::AsyncClient;

pub struct MMqtt {
    config: MMqttConfig,
    /// 发送至core的指令
    core: Sender<ModuleCommand>,
    /// 接收核心的指令
    recv: Receiver<CoreCommand>,
}

impl Module for MMqtt {
    fn start(self) {
        let Self { config, core, recv } = self;
        task::spawn(async move {
            start(config, core, recv).await;
        });
    }
}

async fn start(config: MMqttConfig, core: Sender<ModuleCommand>, recv: Receiver<CoreCommand>) {
    let (client, mut eventloop) = AsyncClient::new(config.into(), 1024);
    loop {
        select! {
            packet = eventloop.poll().fuse() => match packet {
                Ok(_) => {

                }
                Err(_) => {

                }
                // Ok(Event::Incoming(Packet::Publish(packet))) => {
                //     task::spawn(
                //         mqtt_receive(packet, global_param.clone(), mqtt_msg_map.clone())
                //     );
                // },
                // Err(e) => {
                //     //Io(Os { code: 11001, kind: Other, message: "不知道这样的主机。" })
                //     error!("Received = {:?}", e);
                //     task::sleep(Duration::from_secs(10u64)).await;
                // },
                // Ok(Event::Outgoing(_outgoing)) => {
                //     // debug!("{:?}", _outgoing);
                // },
                // Ok(Event::Incoming(Packet::ConnAck(ca))) => {
                //     let ConnAck{session_present: _, code} = ca;
                //     match code {
                //         ConnectReturnCode::Success => {
                //             if let Err(e) = client.subscribe(&subscribe, QoS::ExactlyOnce).await {
                //                 error!("mqtt订阅失败：{:?}", e);
                //             } else {
                //                 info!("mqtt订阅成功！");
                //                 if let Ok(msg) = global_param.get_heart_check_msg() {
                //                     send_req_to_mqtt(global_param.clone(), "heartCheck", msg, None).await;
                //                 }
                //             }
                //         },
                //         e => {
                //             error!("mqtt连接失败：{:?}", e);
                //         }
                //     }
                //
                // },
                // _o => {
                //     // debug!("{:?}", _o);
                // }
            },
            packet = recv.recv().fuse() => match packet {
                Ok(_) => {
                    // let topic = packet.get_topic();
                    // // let topic_tmp = topic.clone();
                    // let payload = packet.get_payload();
                    // debug!("接收需要发送的mqtt数据：{}--{}", topic, payload);
                    // if let Err(err) = client.publish(topic, QoS::ExactlyOnce, false, payload).await {
                    //     warn!("消息：{:?}发送失败：{:?}", packet.get_u(), err);
                    // }
                    // // debug!("mqtt数据发送成功：{}--{}", topic_tmp, payload);
                    // if packet.need_follow_up {
                    //     //TODO 需要后续跟踪清理
                    //     let mut tmp_map = mqtt_msg_map.lock().await;
                    //     tmp_map.insert(packet.get_u(), packet);
                    // }
                },
                Err(e) => {
                    error!("接收需要发送的mqtt数据异常：{:?}", e);
                }
            }
        }
    }
}

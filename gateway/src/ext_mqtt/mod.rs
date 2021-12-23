/// mqtt报文的相关实现
pub mod event;
use crate::event::init_event;
use crate::pub_use::*;
use crate::*;
use rumqttc::Publish;
use std::collections::HashMap;

pub async fn mqtt_receive(
    packet: Publish,
    global: Arc<Global>,
    req_map: Arc<Mutex<HashMap<String, MqttPacket>>>,
) {
    // let may_panic = async {
    if let Err(e) = mqtt_msg_deal(packet, global, req_map.clone()).await {
        error!("{:?}", e);
    }
    // };
    // if let Err(e) = may_panic.catch_unwind().await {
    //     error!("{:?}", e);
    // }
}

async fn mqtt_msg_deal(
    packet: Publish,
    global: Arc<Global>,
    req_map: Arc<Mutex<HashMap<String, MqttPacket>>>,
) -> Result<()> {
    let packet = MqttPacket::from_received_msg(&packet, global.clone())?;
    if packet.is_req() {
        let event = init_event(packet, global.clone()).await?;
        return event.action().await;
    } else {
        if let Some(ru) = packet.get_ru() {
            let tmp_map = req_map.lock().await;
            if let Some(pub_packet) = tmp_map.get(ru.as_str()) {
                if let Some(box_action) = &pub_packet.ack_action {
                    box_action.ack_action().await;
                } else {
                    warn!("需要跟进应答报文，但却没有设置处理方法");
                }
            }
        }
    }
    return Ok(());
}

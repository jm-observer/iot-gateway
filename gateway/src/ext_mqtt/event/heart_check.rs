use crate::*;
use gateway_derive::SuperActionImpl;
#[derive(Debug, SuperActionImpl)]
pub struct HeartCheck {
    packet: MqttPacket,
    global: Arc<Global>,
}

unsafe impl Send for HeartCheck {}
unsafe impl Sync for HeartCheck {}

impl HeartCheck {
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        let hc = HeartCheck {
            packet: packet,
            global,
        };
        let box_ssh = Arc::new(hc);
        Ok(box_ssh)
    }
}
#[async_trait]
impl Action for HeartCheck {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        // self.send_success_ack(Some(ext), None).await;
        self.packet
            .ack_success_packet(Some(self.global.get_heart_check_msg()?), None)
            .await;
        Ok(())
    }
}

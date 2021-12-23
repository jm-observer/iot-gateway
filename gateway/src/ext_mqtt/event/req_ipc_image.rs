use crate::*;
use gateway_derive::SuperActionImpl;
use std::fmt::Debug;
#[derive(Debug, SuperActionImpl)]
pub struct ReqIpcImage {
    pub packet: Arc<MqttPacket>,
    pub global: Arc<Global>,
}

unsafe impl Send for ReqIpcImage {}
unsafe impl Sync for ReqIpcImage {}

impl ReqIpcImage {
    #[allow(dead_code)]
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        // let time = packet.get_msg().ext_get_int_or_default("time", 60f64);
        let hc = ReqIpcImage {
            packet: Arc::new(packet),
            global,
        };
        let box_ssh = Arc::new(hc);
        Ok(box_ssh)
    }
}
#[async_trait]
impl Action for ReqIpcImage {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        if let Err(e) = get_original_image(self.packet.clone()).await {
            self.packet
                .ack_fail_from_failtype(e.to_string().as_str())
                .await;
        }
        Ok(())
    }
}

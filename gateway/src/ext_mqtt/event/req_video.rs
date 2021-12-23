use crate::*;
use gateway_derive::SuperActionImpl;
use std::fmt::Debug;

#[derive(Debug, SuperActionImpl)]
pub struct ReqVideo {
    pub packet: Arc<MqttPacket>,
    pub global: Arc<Global>,
}

unsafe impl Send for ReqVideo {}
unsafe impl Sync for ReqVideo {}

impl ReqVideo {
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        packet.get_msg().ext_get_string("uuid")?;
        // let time = packet.get_msg().ext_get_int_or_default("time", 60f64);
        let hc = ReqVideo {
            packet: Arc::new(packet),
            global,
        };
        let box_ssh = Arc::new(hc);
        Ok(box_ssh)
    }
}
#[async_trait]
impl Action for ReqVideo {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        self.global
            .sender_to_ffmpeg
            .send(VideoCommand::ReqVideo(self.packet.clone()))
            .await?;
        Ok(())
    }
}

use crate::*;
use gateway_derive::SuperActionImpl;
#[derive(Debug, SuperActionImpl)]
pub struct EndVideo {
    pub packet: Arc<MqttPacket>,
    pub global: Arc<Global>,
}

unsafe impl Send for EndVideo {}
unsafe impl Sync for EndVideo {}

impl EndVideo {
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        let hc = EndVideo {
            packet: Arc::new(packet),
            global,
        };
        Ok(Arc::new(hc))
    }
}
#[async_trait]
impl Action for EndVideo {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        self.global
            .sender_to_ffmpeg
            .send(VideoCommand::CloudEndVideo(self.packet.clone()))
            .await?;
        Ok(())
    }
    //
    // fn get_uuid_rtsp(&self) {
    //     self.global.toml_config.read().await.
    // }
}

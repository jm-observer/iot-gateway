use crate::check_device_password_by_cloud;
use crate::*;
use std::fmt::Debug;

#[derive(Debug, SuperActionImpl)]
pub struct AckIpc {
    pub packet: Arc<MqttPacket>,
    pub global: Arc<Global>,
    pub uuid: String,
    pub user: String,
    pub password: String,
}
//
// unsafe impl Send for AckIpc {}
// unsafe impl Sync for AckIpc {}

impl AckIpc {
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        let user = packet.get_msg().ext_get_string("user")?;
        let password = packet.get_msg().ext_get_string("password")?;
        let uuid = packet.get_msg().ext_get_string("uuid")?;
        // let time = packet.get_msg().ext_get_int_or_default("time", 60f64);
        let hc = AckIpc {
            packet: Arc::new(packet),
            global,
            uuid,
            user,
            password,
        };
        let box_ssh = Arc::new(hc);
        Ok(box_ssh)
    }
}
// impl SuperAction for AckIpc {}
#[async_trait]
impl Action for AckIpc {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        match check_device_password_by_cloud(
            &self.user,
            &self.password,
            &self.uuid,
            self.global.clone(),
        )
        .await
        {
            Ok(_) => self.packet.ack_brief_success_packet().await,
            Err(e) => self.packet.ack_fail_from_failtype(&e).await,
        }
        Ok(())
    }
}

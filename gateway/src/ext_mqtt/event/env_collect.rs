use crate::event::env_serial_task_detail;
use crate::*;
use gateway_derive::SuperActionImpl;
use std::fmt::Debug;
#[derive(Debug, SuperActionImpl)]
pub struct EnvCollect {
    pub packet: Arc<MqttPacket>,
    pub global: Arc<Global>,
}

unsafe impl Send for EnvCollect {}
unsafe impl Sync for EnvCollect {}

impl EnvCollect {
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        let hc = EnvCollect {
            packet: Arc::new(packet),
            global,
        };
        let box_ssh = Arc::new(hc);
        Ok(box_ssh)
    }
}
#[async_trait]
impl Action for EnvCollect {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        match env_serial_task_detail(self.global.clone()).await {
            Ok((air, humidity, temp)) => {
                let mut msg = Json::new();
                msg.ext_add_f64_object("airp", get_format_f64(air))
                    .ext_add_f64_object("humidity", get_format_f64(humidity))
                    .ext_add_f64_object("temp", get_format_f64(temp));
                self.packet.ack_success_packet(Some(msg), None).await;
            }
            Err(e) => {
                self.packet
                    .ack_fail_from_failtype(e.to_string().as_str())
                    .await;
            }
        }
        Ok(())
    }
}

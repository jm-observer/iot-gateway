use crate::*;
use std::fmt::Debug;
use crate::event::serial::ele_serial_task_detail;

#[derive(Debug, SuperActionImpl)]
pub struct EleCollect {
    pub packet: Arc<MqttPacket>,
    pub global: Arc<Global>,
}

unsafe impl Send for EleCollect {}
unsafe impl Sync for EleCollect {}

impl EleCollect {
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        // let time = packet.get_msg().ext_get_int_or_default("time", 60f64);
        let hc = EleCollect {
            packet: Arc::new(packet),
            global,
        };
        let box_ssh = Arc::new(hc);
        Ok(box_ssh)
    }
}
#[async_trait]
impl Action for EleCollect {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        match ele_serial_task_detail(self.global.clone()).await {
            Ok((va, vb, vc, vd)) => {
                //上报
                let mut msg = Json::new();
                msg.ext_add_f64_object("va", get_format_f64(va))
                    .ext_add_f64_object("vb", get_format_f64(vb))
                    .ext_add_f64_object("vc", get_format_f64(vc))
                    .ext_add_f64_object("vd", get_format_f64(vd));
                self.packet.ack_success_packet(Some(msg), None).await;
            }
            Err(e) => {
                self.packet.ack_fail_from_failtype(&e).await;
            }
        }
        Ok(())
    }
}

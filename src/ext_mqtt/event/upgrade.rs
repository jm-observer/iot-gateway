use crate::*;
use std::process::Command;

#[derive(Debug, SuperActionImpl)]
pub struct Upgrade {
    packet: MqttPacket,
    global: Arc<Global>,
    file: String,
    key: String,
    version: String,
}

unsafe impl Send for Upgrade {}
unsafe impl Sync for Upgrade {}

impl Upgrade {
    pub fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        let msg = packet.get_msg();
        let file = msg.ext_get_string("file")?;
        let key = msg.ext_get_string("key")?;
        let version = msg.ext_get_string("version")?;
        let hc = Upgrade {
            packet: packet,
            global,
            file,
            key,
            version,
        };
        let box_ssh = Arc::new(hc);
        Ok(box_ssh)
    }
}
#[async_trait]
impl Action for Upgrade {
    /**
     * 1. 版本比对
     * 2. 文件下载至临时文件夹；3. 新程序检查(新版本没有这个吧？？)；4. 移动至正式文件夹
     */
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        let ext = Json::new();
        if self.version == self.global.get_version() {
            self.packet
                .ack_fail_packet(None, None, Some("已是当前版本，无须升级"))
                .await;
            // if let Err(err) = self
            //     .send_fail_ack("已是当前版本，无须升级".to_string())
            //     .await
            // {
            //     error!("{:?}", err);
            // }
            return Ok(());
        } else {
            let res = get_image("/opt/iot", "iot_gateway_tmp", &self.key).await;
            deal_file("/opt/iot/iot_gateway_tmp", "/opt/iot/iot_gateway")?;
        }
        self.packet
            .ack_success_packet(None, Some("下载成功，准备重启..."))
            .await;
        if let Err(com) = Command::new("sudo")
            .arg("systemctl")
            .arg("restart")
            .arg("iotgateway")
            .status()
        {
            error!("{:?}", com);
            return fail(com.to_string());
        }
        return Ok(());
    }
}

use crate::cos::CosTask;
/// 全局变量
///
use crate::*;
use async_std::channel::bounded;
use async_std::sync::RwLock;
use regex::Regex;
use std::collections::HashMap;

#[cfg(feature = "biz")]
pub const TYPE: &str = "biz";
#[cfg(feature = "support")]
pub const TYPE: &str = "support";

#[derive(Debug)]
pub struct Global {
    // config: Json,
    mac_addr: String,
    pub mqtt_id: String,
    pub topic_regex: Regex,
    pub sender_to_mqtt: Sender<MqttPacket>,
    pub rec_from_other: Receiver<MqttPacket>,
    pub toml_config: RwLock<Toml>,
    pub ipc_ips: RwLock<HashMap<String, String>>,
    pub ipc_ips_tmp: RwLock<HashMap<String, String>>,

    pub ipc_addr: RwLock<HashMap<String, String>>,
    pub ipc_addr_tmp: RwLock<HashMap<String, String>>,
    //用于发送请求、停止视频传输
    pub sender_to_ffmpeg: Sender<VideoCommand>,
    pub rec_video_command: Receiver<VideoCommand>,
    // 用于发送NodeMcu指令
    pub nodemcu_command: (Sender<VideoCommand>, Receiver<VideoCommand>),
    pub sender_to_ssh: Sender<TargetSsh>,
    pub rec_ssh: Receiver<TargetSsh>,
    pub sender_to_cos: Sender<CosTask>,
    pub rec_cos: Receiver<CosTask>,
}
//Receiver<TargetSsh>
pub enum VideoCommand {
    ReqVideo(Arc<MqttPacket>),
    CloudEndVideo(Arc<MqttPacket>),
    ReqImages(Arc<MqttPacket>),
}

impl Global {
    // pub fn get_config(&self) -> &Json {
    //     &self.config
    // }
    pub fn get_mac_addr(&self) -> String {
        self.mac_addr.clone()
    }
    pub fn get_version(&self) -> String {
        return env!("CARGO_PKG_VERSION").to_string();
    }
    pub fn get_local_path(&self) -> String {
        return env!("CARGO_PKG_VERSION").to_string();
    }

    pub async fn clear_ipc_ips_tmp(&self) {
        self.ipc_ips_tmp.write().await.clear();
    }
    pub async fn update_ipc_ips_tmp(&self, key: &str, val: &str) {
        self.ipc_ips_tmp
            .write()
            .await
            .insert(key.to_string(), val.to_string());
    }
    pub async fn update_ipc_ips(&self) {
        let mut ips = self.ipc_ips.write().await;
        ips.clear();
        let ips_tmp = self.ipc_ips_tmp.read().await;
        for (key, val) in ips_tmp.iter() {
            ips.insert(key.to_string(), val.to_string());
        }
    }

    pub async fn clear_ipc_addr_tmp(&self) {
        self.ipc_addr_tmp.write().await.clear();
    }
    pub async fn update_ipc_addr_tmp(&self, key: &str, val: &str) {
        self.ipc_addr_tmp
            .write()
            .await
            .insert(key.to_string(), val.to_string());
    }
    pub async fn update_ipc_addrs(&self) {
        let mut ips = self.ipc_addr.write().await;
        ips.clear();
        let ips_tmp = self.ipc_addr_tmp.read().await;
        for (key, val) in ips_tmp.iter() {
            ips.insert(key.to_string(), val.to_string());
        }
        // debug!("ips={:?}", ips);
        // debug!("ips_tmp{:?}", ips_tmp);
    }

    pub async fn send_to_mqtt(&self, packet: MqttPacket) {
        // debug!("{:?}", packet);
        if let Err(e) = self.sender_to_mqtt.send(packet).await {
            error!("{:?}", e);
        }
    }

    pub async fn get_two_level_string(&self, class: &str, key: &str) -> Result<String> {
        self.toml_config
            .read()
            .await
            .get_two_level_string(class, key)
    }
    pub async fn get_two_level_f64(&self, class: &str, key: &str) -> Result<f64> {
        self.toml_config.read().await.get_two_level_f64(class, key)
    }
    pub async fn get_two_level_int(&self, class: &str, key: &str) -> Result<i64> {
        self.toml_config.read().await.get_two_level_int(class, key)
    }

    pub async fn update_two_level_string(&self, class: &str, key: &str, val: &str) {
        self.toml_config
            .write()
            .await
            .update_two_level_string(class, key, val)
            .await;
    }
    pub async fn update_two_level_f64(&mut self, class: &str, key: &str, val: f64) {
        self.toml_config
            .write()
            .await
            .update_two_level_f64(class, key, val)
            .await;
    }
    pub async fn update_two_level_int(&mut self, class: &str, key: &str, val: i64) {
        self.toml_config
            .write()
            .await
            .update_two_level_int(class, key, val)
            .await;
    }

    pub fn get_heart_check_msg(&self) -> Result<Json> {
        let mut ext = Json::new();
        ext.ext_add_string_object("v", self.get_version());
        ext.ext_add_string_object("ip", get_ip()?.to_string());
        ext.ext_add_str_object("type", TYPE);
        Ok(ext)
    }
}

pub async fn send_req_to_mqtt(global: Arc<Global>, event: &str, msg: Json, rp: Option<Json>) {
    // debug!("{:?}", packet);
    let packet = MqttPacket::new_req_packet(event, msg, global.clone(), rp);
    global.send_to_mqtt(packet).await;
}

#[allow(dead_code)]
pub async fn report_info_msg(global: Arc<Global>, msg: &str) {
    _report_info_msg(global, "info", msg).await;
}
#[allow(dead_code)]
pub async fn report_warn_msg(global: Arc<Global>, msg: &str) {
    _report_info_msg(global, "warn", msg).await;
}
#[allow(dead_code)]
pub async fn report_error_msg(global: Arc<Global>, msg: &str) {
    _report_info_msg(global, "error", msg).await;
}
async fn _report_info_msg(global: Arc<Global>, level: &str, msg: &str) {
    let mut body = Json::new();
    body.ext_add_str_object("l", level)
        .ext_add_str_object("i", msg);
    let packet = MqttPacket::new_req_packet("reportinfo", body, global.clone(), None);
    if let Err(e) = global.sender_to_mqtt.send(packet).await {
        error!("{:?}", e);
    }
}

pub fn init_config() -> Result<Arc<Global>> {
    let mac_addr = get_mac_address()?;
    let (sender_to_mqtt, rec_from_other) = bounded::<MqttPacket>(1024);
    let (sender_to_ffmpeg, rec_video_command) = bounded::<VideoCommand>(1024);
    let (sender_to_ssh, rec_ssh) = bounded::<TargetSsh>(1024);
    let (sender_to_cos, rec_cos) = bounded::<CosTask>(1024);

    let nodemcu_command = bounded::<VideoCommand>(1024);

    let toml_config = init_toml_config()?;
    let mut mqtt_id = mac_addr.clone();
    mqtt_id.push_str(&toml_config.get_two_level_string("mqtt", "id")?);
    // mqtt_id.push_str("_support");

    // let config = init_json_config();
    let config = Global {
        // config,
        mac_addr,
        mqtt_id,
        topic_regex: Regex::new(r"^/([^/]+)/([^/]+)/([^/]+)/([^/]+)/([^/]+)$").unwrap(),
        sender_to_mqtt,
        rec_from_other,
        toml_config: RwLock::new(toml_config),
        ipc_ips: RwLock::new(HashMap::<String, String>::with_capacity(20)),
        ipc_ips_tmp: RwLock::new(HashMap::<String, String>::with_capacity(20)),
        ipc_addr: RwLock::new(HashMap::<String, String>::with_capacity(20)),
        ipc_addr_tmp: RwLock::new(HashMap::<String, String>::with_capacity(20)),
        sender_to_ffmpeg,
        rec_video_command,
        nodemcu_command: nodemcu_command,
        sender_to_ssh,
        rec_ssh,
        sender_to_cos,
        rec_cos,
    };
    //todo 初始化路径
    debug!("初始化global完成");
    Ok(Arc::new(config))
}

/// mqtt报文的相关操作
///
///
use crate::*;
use async_std::sync::Arc;
use rumqttc::Publish;
use std::fmt::{Debug, Display};
use std::{fmt, str};

pub trait SuperAction {
    fn _get_global(&self) -> Arc<Global>;
    fn _get_packet(&self) -> &MqttPacket;
}

#[async_trait]
pub trait Action: SuperAction + Debug + Send + Sync {
    async fn action(&self) -> Result<()> {
        if let Err(e) = self.detail_action().await {
            self._get_packet()
                .ack_fail_packet(None, None, Some(&format!("{:?}", e)))
                .await;
        }
        Ok(())
    }

    async fn detail_action(&self) -> Result<()>;
    //
    // fn _get_global(&self) -> Arc<Global>;
    // fn _get_packet(&self) -> &MqttPacket;
}
// impl dyn Action {}

#[async_trait]
pub trait AckAction: SuperAction + Debug + Send + Sync {
    async fn ack_action(&self);
}

#[derive(Debug)]
pub struct MqttPacket {
    head: PacketHeader,
    msg: Json,
    pub need_follow_up: bool,
    pub ack_action: Option<Box<dyn AckAction>>,
    pub global: Arc<Global>,
}

// impl Deref for MqttPacket {
//     type Target = Json;
//     fn deref(&self) -> &Json {
//         &self.msg
//     }
// }

impl MqttPacket {
    pub fn new_req_packet(
        event: &str,
        msg: Json,
        global: Arc<Global>,
        rp: Option<Json>,
    ) -> MqttPacket {
        MqttPacket {
            head: PacketHeader::new(event, global.mqtt_id.clone(), rp),
            msg,
            need_follow_up: false,
            ack_action: None,
            global,
        }
    }

    #[allow(unused_variables)]
    pub async fn ack_success_packet(&self, ext: Option<Json>, suc_msg: Option<&str>) {
        let mut msg = Json::new();
        msg.ext_add_str_object(MQTT_ACK_CODE, MQTT_ACK_CODE_OK);
        msg.ext_add_str_object(MQTT_ACK_MSG, suc_msg.unwrap_or("执行成功"));
        if let Some(ext) = ext {
            msg.ext_add_all(ext);
        }
        match self.general_ack_packet(msg) {
            Ok(ack) => self.global.send_to_mqtt(ack).await,
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
    #[allow(unused_variables)]
    pub async fn ack_brief_success_packet(&self) {
        let mut msg = Json::new();
        msg.ext_add_str_object(MQTT_ACK_CODE, MQTT_ACK_CODE_OK)
            .ext_add_str_object(MQTT_ACK_MSG, "执行成功");
        match self.general_ack_packet(msg) {
            Ok(ack) => self.global.send_to_mqtt(ack).await,
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
    #[allow(unused_variables)]
    pub async fn ack_fail_packet(
        &self,
        ext: Option<Json>,
        fail_code: Option<&str>,
        fail_msg: Option<&str>,
    ) {
        let mut msg = Json::new();
        msg.ext_add_str_object(MQTT_ACK_CODE, fail_code.unwrap_or(MQTT_ACK_CODE_ERR));
        msg.ext_add_str_object(MQTT_ACK_MSG, fail_msg.unwrap_or("执行失败"));
        if let Some(ext) = ext {
            msg.ext_add_all(ext);
        }
        match self.general_ack_packet(msg) {
            Ok(ack) => self.global.send_to_mqtt(ack).await,
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
    #[allow(unused_variables)]
    pub async fn ack_fail_from_failtype(&self, fail_type: &FailTypeEnum) {
        let mut msg = Json::new();
        msg.ext_add_str_object(MQTT_ACK_CODE, "1000")
            .ext_add_str_object(MQTT_ACK_MSG, &fail_type.to_string());
        match self.general_ack_packet(msg) {
            Ok(ack) => self.global.send_to_mqtt(ack).await,
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
    pub fn from_received_msg(packet: &Publish, global: Arc<Global>) -> Result<Self> {
        let msg = &packet.payload;
        let payload = str::from_utf8(msg).unwrap();
        debug!("开始处理mqtt消息：topic={} msg={:?}", packet.topic, payload);
        let msg = Json::parse(msg)?;
        let topic = &packet.topic;
        Ok(MqttPacket {
            head: PacketHeader::from_rec(topic, &msg, &global.clone())?,
            msg,
            need_follow_up: false,
            ack_action: None,
            global,
        })
    }
    pub fn general_ack_packet(&self, msg: Json) -> Result<MqttPacket> {
        Ok(MqttPacket {
            head: self.head.general_ack_header()?,
            msg,
            need_follow_up: false,
            ack_action: None,
            global: self.global.clone(),
        })
    }
    pub fn get_msg(&self) -> &Json {
        &self.msg
    }
    pub fn get_topic(&self) -> String {
        self.head.general_topic()
    }
    pub fn get_payload(&self) -> String {
        self.head.get_all_msg_data(self.msg.clone()).print()
    }
    pub fn is_req(&self) -> bool {
        self.head.packet_type.is_req()
    }
    pub fn get_ru(&self) -> Option<String> {
        self.head.ru.clone()
    }
    pub fn get_u(&self) -> String {
        self.head.u.clone()
    }
    pub fn get_event(&self) -> &str {
        self.head.get_event()
    }
}

#[derive(Debug)]
pub enum MqttPacketType {
    Req,
    Ack,
}

impl MqttPacketType {
    pub fn new(packet_type: &str) -> MqttPacketType {
        match packet_type {
            "req" => MqttPacketType::Req,
            _ => MqttPacketType::Ack,
        }
    }
    pub fn is_req(&self) -> bool {
        match self {
            MqttPacketType::Req => true,
            _ => false,
        }
    }
}

impl Display for MqttPacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttPacketType::Req => {
                write!(f, "req")
            }
            MqttPacketType::Ack => {
                write!(f, "ack")
            }
        }
    }
}

#[derive(Debug)]
pub struct PacketHeader {
    u: String,
    packet_type: MqttPacketType,
    rec_id: String,
    from_id: String,
    from_type: String,
    event: String,
    pub ru: Option<String>,
    rp: Option<Json>,
}

fn get_clone_option_json(json: &Option<Json>) -> Option<Json> {
    if let Some(json) = json {
        Some(json.clone())
    } else {
        None
    }
}

impl PacketHeader {
    pub fn general_ack_header(&self) -> Result<Self> {
        if self.packet_type.is_req() {
            Ok(PacketHeader {
                u: uuid(),
                packet_type: MqttPacketType::Req,
                rec_id: self.from_id.clone(),
                from_id: self.rec_id.clone(),
                from_type: MQTT_SELF_TYPE.to_string(),
                event: self.event.clone(),
                ru: Some(self.u.clone()),
                rp: get_clone_option_json(&self.rp),
            })
        } else {
            fail_from_str("该报文为响应报文，无法再生成对应响应报文")
        }
    }
    pub fn get_all_msg_data(&self, mut msg: Json) -> Json {
        msg.ext_add_str_object("u", self.u.as_str());
        if let Some(ref ru) = self.ru {
            msg.ext_add_str_object("ru", ru.as_str());
        }
        if let Some(ref rp) = self.rp {
            msg.ext_add_all(rp.clone());
        }
        msg
    }
    pub fn general_topic(&self) -> String {
        format!(
            "/{}/{}/{}/{}/{}",
            self.rec_id, self.packet_type, self.from_type, self.from_id, self.event
        )
    }
    pub fn get_event(&self) -> &str {
        &self.event
    }
    pub fn new(event: &str, mqtt_id: String, rp: Option<Json>) -> Self {
        PacketHeader {
            u: uuid(),
            packet_type: MqttPacketType::Req,
            rec_id: MQTT_SERVER.to_string(),
            from_id: mqtt_id,
            from_type: MQTT_SELF_TYPE.to_string(),
            event: event.to_string(),
            ru: None,
            rp,
        }
    }
    pub fn from_rec(topic: &String, msg: &Json, global: &Arc<Global>) -> Result<Self> {
        let (rec_id, packet_type, from_type, from_id, event) =
            analysis_topic(topic, global.clone())?;
        let ru = msg
            .ext_get_string("ru")
            .map_or_else(|_| None, |val| Some(val));
        return Ok(PacketHeader {
            u: msg.ext_get_str_or_default("u", ""),
            packet_type: MqttPacketType::new(packet_type.as_str()),
            rec_id,
            from_id,
            from_type,
            event,
            ru,
            rp: msg.ext_get("rp"),
        });
    }
}

fn analysis_topic(
    topic: &str,
    global: Arc<Global>,
) -> Result<(String, String, String, String, String)> {
    if let Some(i) = global.topic_regex.captures(topic) {
        //_, packet_type, from_type, from_id, event
        let rec_id = get_index_group_str(&i, 1usize)?;
        let packet_type = get_index_group_str(&i, 2usize)?;
        let from_type = get_index_group_str(&i, 3usize)?;
        let from_id = get_index_group_str(&i, 4usize)?;
        let event = get_index_group_str(&i, 5usize)?;
        return Ok((rec_id, packet_type, from_type, from_id, event));
    }
    fail(format!("mqtt主题（{}）不匹配！", topic).to_string())
}

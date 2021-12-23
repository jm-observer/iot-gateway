#[cfg(feature = "dev")]
pub const MQTT_SERVER: &str = "sbiot_dev";

#[cfg(not(feature = "dev"))]
pub const MQTT_SERVER: &str = "sbiot";

pub const MQTT_SELF_TYPE: &str = "netgate";
pub const MQTT_ACK_CODE: &str = "ExcResultCode";
pub const MQTT_ACK_MSG: &str = "ExcResultMsg";
pub const MQTT_ACK_CODE_OK: &str = "0000";
pub const MQTT_ACK_CODE_ERR: &str = "1000";

pub const ONVIF_DISCOVERY0: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing"><s:Header><a:Action s:mustUnderstand="1">http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</a:Action><a:MessageID>uuid:"#;
pub const ONVIF_DISCOVERY1: &str = r#"</a:MessageID><a:ReplyTo><a:Address>http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous</a:Address></a:ReplyTo><a:To s:mustUnderstand="1">urn:schemas-xmlsoap-org:ws:2005:04:discovery</a:To></s:Header><s:Body><Probe xmlns="http://schemas.xmlsoap.org/ws/2005/04/discovery"><d:Types xmlns:d="http://schemas.xmlsoap.org/ws/2005/04/discovery" xmlns:dp0="http://www.onvif.org/ver10/network/wsdl">dp0:NetworkVideoTransmitter</d:Types></Probe></s:Body></s:Envelope>"#;

pub const REQUEST01: &str = r#"<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd" xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"><env:Header>"#;

pub const AUTH_HEARD01: &str = "<wsse:Security><wsse:UsernameToken><wsse:Username>";
pub const AUTH_HEARD02: &str = r#"</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest">"#;
pub const AUTH_HEARD03: &str = r#"</wsse:Password><wsse:Nonce EncodingType="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary">"#;
pub const AUTH_HEARD04: &str = r#"</wsse:Nonce><wsu:Created>"#;
pub const AUTH_HEARD05: &str = r#"</wsu:Created></wsse:UsernameToken></wsse:Security>"#;

pub const REQUEST02: &str = r#"</env:Header><env:Body>"#;

pub const GET_DEVICEINFORMATION: &str =
    r#"<GetDeviceInformation xmlns="http://www.onvif.org/ver10/device/wsdl"/>"#;
pub const GET_CAPABILITIES: &str =
    r#"<GetCapabilities xmlns="http://www.onvif.org/ver10/device/wsdl"/>"#;

pub const GET_SERVICES: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema"><GetServices xmlns="http://www.onvif.org/ver10/device/wsdl"><IncludeCapability>false</IncludeCapability></GetServices></s:Body></s:Envelope>"#;

pub const GET_PROFILES: &str = r#"</env:Header><env:Body><GetProfiles xmlns="http://www.onvif.org/ver10/media/wsdl"/></env:Body></env:Envelope>"#;

pub const GET_STREAM_URL0: &str = r#"</env:Header><env:Body><GetStreamUri xmlns="http://www.onvif.org/ver10/media/wsdl"><StreamSetup>
<Stream xmlns="http://www.onvif.org/ver10/schema">RTP-Unicast</Stream>
<Transport xmlns="http://www.onvif.org/ver10/schema"><Protocol>UDP</Protocol></Transport></StreamSetup>
<ProfileToken>"#;
pub const GET_STREAM_URL1: &str = r#"</ProfileToken></GetStreamUri></env:Body></env:Envelope>"#;
pub const REQUEST04: &str = r#"</env:Body></env:Envelope>"#;

//新发现ipc，且无法认证
pub const ONVIF_IPC_STATUS_NEW_UNAUTH: &str = "new_unauth";
//原先已认证ipc，当下无法认证
pub const ONVIF_IPC_STATUS_UPDATE_UNAUTH: &str = "update_unauth";
//原先已认证ipc，当下重新认证（原用户信息失效）
pub const ONVIF_IPC_STATUS_UPDATE_AUTH: &str = "update_auth";
//原先已认证ipc，当前在线
pub const ONVIF_IPC_STATUS_ONLINE: &str = "online";
//新发现ipc，且认证成功
pub const ONVIF_IPC_STATUS_ADD: &str = "add";
//原先已认证ipc，当前离线中
pub const ONVIF_IPC_STATUS_OFFLINE: &str = "offline";

#[derive(PartialEq)]
pub enum IpcStatus {
    NewUnauth,
    UpdateUnatuth,
    UpdateAuth,
    Online,
    Add,
    Offline,
}

impl IpcStatus {
    pub fn to_string(&self) -> String {
        match self {
            IpcStatus::NewUnauth => ONVIF_IPC_STATUS_NEW_UNAUTH.to_string(),
            IpcStatus::UpdateUnatuth => ONVIF_IPC_STATUS_UPDATE_UNAUTH.to_string(),
            IpcStatus::UpdateAuth => ONVIF_IPC_STATUS_UPDATE_AUTH.to_string(),
            IpcStatus::Online => ONVIF_IPC_STATUS_ONLINE.to_string(),
            IpcStatus::Add => ONVIF_IPC_STATUS_ADD.to_string(),
            IpcStatus::Offline => ONVIF_IPC_STATUS_OFFLINE.to_string(),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            IpcStatus::NewUnauth => ONVIF_IPC_STATUS_NEW_UNAUTH,
            IpcStatus::UpdateUnatuth => ONVIF_IPC_STATUS_UPDATE_UNAUTH,
            IpcStatus::UpdateAuth => ONVIF_IPC_STATUS_UPDATE_AUTH,
            IpcStatus::Online => ONVIF_IPC_STATUS_ONLINE,
            IpcStatus::Add => ONVIF_IPC_STATUS_ADD,
            IpcStatus::Offline => ONVIF_IPC_STATUS_OFFLINE,
        }
    }
}

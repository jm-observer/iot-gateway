/// 相机的onvif的实现
use crate::*;
use async_std::net::UdpSocket;
use chrono::Utc;
use mio_httpc::CallBuilder;
use regex::Regex;
use roxmltree::{Document, Node};
use sha1::Sha1;
use std::collections::HashMap;
use std::time::Duration;

const IPC_IP: &str = r"^http://([^/]+)/onvif/device_service$";
const IPC_UUID: &str = r"^.*uuid:(.*)$";

struct Ipc {
    pub uuid: String,
    pub addr: String,
    pub ip: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub stream_url: Option<String>,
    pub manufacturer: Option<String>,
    pub status: IpcStatus,
}

impl Ipc {
    fn new(uuid: &str, addr: &str, ip: &str) -> Self {
        Ipc {
            uuid: uuid.to_string(),
            addr: addr.to_string(),
            ip: ip.to_string(),
            user: None,
            password: None,
            stream_url: None,
            manufacturer: None,
            status: IpcStatus::NewUnauth,
        }
    }
}

async fn get_ipc(global: Arc<Global>, uuid: &str, addr: &str, ip: &str) -> Result<Ipc> {
    let mut ipc = Ipc::new(uuid, addr, ip);
    if let Ok((_, user, password, stream_url, manufacturer)) = global
        .toml_config
        .read()
        .await
        .get_ipc_data(ipc.uuid.as_str())
    {
        ipc.user = Some(user);
        ipc.password = Some(password);
        ipc.stream_url = Some(stream_url);
        ipc.manufacturer = Some(manufacturer);
        ipc.status = IpcStatus::Online
    }
    match ipc.status {
        IpcStatus::NewUnauth => {
            _deal_ipc_new_unauth(&mut ipc, global, addr, ip).await?;
        }
        IpcStatus::Online => {
            _deal_ipc_online(&mut ipc, global.clone(), addr).await?;
        }
        _ => {}
    }
    Ok(ipc)
}

async fn _deal_ipc_online(ipc: &mut Ipc, global: Arc<Global>, addr: &str) -> Result<()> {
    //再次验证用户密码
    if let Ok(_) = _check_device_password(
        ipc.user.as_ref().unwrap().as_str(),
        ipc.password.as_ref().unwrap().as_str(),
        addr,
    ) {
        return Ok(());
    }
    //验证用户密码库
    if let Ok((user, password, _)) = _check_device_passwords(addr, global.clone()).await {
        ipc.status = IpcStatus::UpdateAuth;
        ipc.user = Some(user);
        ipc.password = Some(password);
    }
    //未知用户密码
    ipc.status = IpcStatus::UpdateUnatuth;
    ipc.user = None;
    ipc.password = None;
    ipc.manufacturer = None;
    Ok(())
}
async fn _deal_ipc_new_unauth(
    ipc: &mut Ipc,
    global: Arc<Global>,
    addr: &str,
    ip: &str,
) -> Result<()> {
    if let Ok((user, password, manufacturer)) = _check_device_passwords(addr, global.clone()).await
    {
        ipc.status = IpcStatus::Add;
        // 获取基础数据，保存至配置，更新本地数据
        let url = format!("http://{}/onvif/media_service", ip.clone());
        let profile = get_profiles(user.as_str(), password.as_str(), &url)?;
        let stream_url = get_stream_url_xml(
            user.as_str(),
            password.as_str(),
            profile.name.as_str(),
            url.as_str(),
        )?;
        ipc.stream_url = Some(stream_url);
        ipc.user = Some(user);
        ipc.password = Some(password);
        ipc.manufacturer = Some(manufacturer);
    }
    Ok(())
}

///
/// 1. 定时更新ipc的ip地址和用户密码：可认证则更新ip，不可认证则删除本地数据(ip,config)且上报
/// 2. 定时发现新的ipc：可认证则上报并维护本地数据(ip,config)；无法认证仅上报
///
pub async fn onvif_discovery_new(global: Arc<Global>) -> Result<()> {
    let (s, r) = async_std::channel::unbounded::<String>();
    let mut ip = get_ip()?;
    ip.push_str(":0");
    debug!("开始onvif_discovery");
    let client = UdpSocket::bind(ip.as_str()).await?;
    let rec_task = task::spawn(async move {
        let mut request_xml = ONVIF_DISCOVERY0.to_string();
        let id = uuid::Uuid::new_v4().to_string();
        request_xml.push_str(&id);
        request_xml.push_str(ONVIF_DISCOVERY1);
        let mut buf = vec![0; 1024 * 3];
        if let Err(err) = client
            .send_to(request_xml.as_bytes(), "239.255.255.250:3702")
            .await
        {
            error!("{:?}", err);
        }
        loop {
            let (n, _) = client.recv_from(&mut buf).await.unwrap();
            let msg = String::from_utf8(buf[0..n].to_vec()).unwrap();
            if let Err(err) = s.send(msg).await {
                error!("{:?}", err);
            }
        }
    });
    task::sleep(Duration::from_millis(1500)).await;
    rec_task.cancel().await;

    let reg = Regex::new(IPC_IP).unwrap();
    let reg_uuid = Regex::new(IPC_UUID).unwrap();
    let mut ipcs = HashMap::<String, &str>::new();
    global.clear_ipc_ips_tmp().await;
    global.clear_ipc_addr_tmp().await;
    let mut ipc_vec = Json::ARRAY(Vec::<Json>::with_capacity(10));
    while let Ok(res) = r.recv().await {
        if let Ok((uuid, addr, ip)) = get_ipc_uuid_addr(res, &reg, &reg_uuid) {
            if let Ok(ipc) =
                get_ipc(global.clone(), uuid.as_str(), addr.as_str(), ip.as_str()).await
            {
                //无论鉴权是否成功，都应该缓存该数据
                global.update_ipc_ips_tmp(&uuid, &ip).await;
                global.update_ipc_addr_tmp(&uuid, &addr).await;
                ipcs.insert(uuid.clone(), ipc.status.to_str());
                let mut tmp = Json::new();
                tmp.ext_add_str_object("uuid", uuid.as_str());
                tmp.ext_add_str_object("status", ipc.status.to_str());
                match ipc.status {
                    IpcStatus::Add => {
                        tmp.ext_add_str_object("user", ipc.user.unwrap().as_str());
                        tmp.ext_add_str_object("password", ipc.password.unwrap().as_str());
                        tmp.ext_add_str_object("manufacturer", ipc.manufacturer.unwrap().as_str());
                    }
                    _ => {}
                }
                ipc_vec.add(tmp);
            }
        }
    }
    global.update_ipc_ips().await;
    global.update_ipc_addrs().await;
    let config_ipcs = global.toml_config.read().await.get_ipcs()?;
    for ipc in config_ipcs {
        if !ipcs.contains_key(ipc.as_str()) {
            ipcs.insert(ipc.clone(), IpcStatus::Offline.to_str());
        }
    }
    let msg = Json::OBJECT {
        name: "ipclist".to_string(),
        value: Box::new(ipc_vec),
    };
    let mut mqtt_msg = Json::new();
    mqtt_msg.add(msg);
    global
        .send_to_mqtt(MqttPacket::new_req_packet(
            "ipclist",
            mqtt_msg,
            global.clone(),
            None,
        ))
        .await;
    debug!("onvif discover end!");
    Ok(())
}

///
/// 1. 定时更新ipc的ip地址和用户密码：可认证则更新ip，不可认证则删除本地数据(ip,config)且上报
/// 2. 定时发现新的ipc：可认证则上报并维护本地数据(ip,config)；无法认证仅上报
///
pub async fn onvif_discovery(global: Arc<Global>) -> Result<()> {
    let (s, r) = async_std::channel::unbounded::<String>();
    let mut ip = get_ip()?;
    ip.push_str(":0");
    debug!("开始onvif_discovery");
    let client = UdpSocket::bind(ip.as_str()).await?;
    let rec_task = task::spawn(async move {
        let mut request_xml = ONVIF_DISCOVERY0.to_string();
        let id = uuid::Uuid::new_v4().to_string();
        request_xml.push_str(&id);
        request_xml.push_str(ONVIF_DISCOVERY1);
        let mut buf = vec![0; 1024 * 3];
        if let Err(err) = client
            .send_to(request_xml.as_bytes(), "239.255.255.250:3702")
            .await
        {
            error!("{:?}", err);
        }
        loop {
            let (n, _) = client.recv_from(&mut buf).await.unwrap();
            let msg = String::from_utf8(buf[0..n].to_vec()).unwrap();
            if let Err(err) = s.send(msg).await {
                error!("{:?}", err);
            }
        }
    });
    task::sleep(Duration::from_millis(1500)).await;
    rec_task.cancel().await;

    let reg = Regex::new(IPC_IP).unwrap();
    let reg_uuid = Regex::new(IPC_UUID).unwrap();
    let mut ipcs = HashMap::<String, &str>::new();
    global.clear_ipc_ips_tmp().await;
    global.clear_ipc_addr_tmp().await;
    let mut ipc_vec = Json::ARRAY(Vec::<Json>::with_capacity(10));
    while let Ok(res) = r.recv().await {
        if let Ok((uuid, addr, ip)) = get_ipc_uuid_addr(res, &reg, &reg_uuid) {
            //无论鉴权是否成功，都应该缓存该数据
            global.update_ipc_ips_tmp(&uuid, &ip).await;
            global.update_ipc_addr_tmp(&uuid, &addr).await;
            match check_ipc(global.clone(), uuid.as_str(), addr.as_str(), ip.as_str()).await {
                Ok((status, info)) => {
                    ipcs.insert(uuid.clone(), status.to_str());
                    let mut tmp = Json::new();
                    tmp.ext_add_str_object("uuid", uuid.as_str());
                    tmp.ext_add_str_object("status", status.to_str());
                    match status {
                        IpcStatus::Add => {
                            if let Some((user, password, manufacturer)) = info {
                                tmp.ext_add_str_object("user", user.as_str());
                                tmp.ext_add_str_object("password", password.as_str());
                                tmp.ext_add_str_object("manufacturer", manufacturer.as_str());
                            }
                        }
                        _ => {}
                    }
                    ipc_vec.add(tmp);
                }
                Err(e) => {
                    let err_msg = format!("ipc[{}]认证出错：{:?}", uuid.as_str(), e);
                    error!("{}", err_msg);
                    report_error_msg(global.clone(), &err_msg).await;
                }
            }
        }
    }
    global.update_ipc_ips().await;
    global.update_ipc_addrs().await;
    let config_ipcs = global.toml_config.read().await.get_ipcs()?;
    for ipc in config_ipcs {
        if !ipcs.contains_key(ipc.as_str()) {
            ipcs.insert(ipc.clone(), IpcStatus::Offline.to_str());
        }
    }

    let msg = Json::OBJECT {
        name: "ipclist".to_string(),
        value: Box::new(ipc_vec),
    };
    let mut mqtt_msg = Json::new();
    mqtt_msg.add(msg);
    global
        .send_to_mqtt(MqttPacket::new_req_packet(
            "ipclist",
            mqtt_msg,
            global.clone(),
            None,
        ))
        .await;
    debug!("onvif discover end!");
    Ok(())
}

fn get_ipc_uuid_addr(
    res: String,
    reg: &Regex,
    reg_uuid: &Regex,
) -> Result<(String, String, String)> {
    let doc = roxmltree::Document::parse(res.as_str())?;
    let mut uuid = _get_node_text("Address", &doc)?;
    if let Some(cap) = reg_uuid.captures(uuid.as_str()) {
        uuid = get_index_group_str(&cap, 1)?;
    }
    let addr = _get_node_text("XAddrs", &doc)?;
    let cap = fail_from_option(
        reg.captures(addr.as_str()),
        &format!("XAddrs[{}]地址不规范", addr),
    )?;
    let ip = get_index_group_str(&cap, 1)?;
    Ok((uuid, addr, ip))
}
///认证ipc
async fn check_ipc(
    global: Arc<Global>,
    uuid: &str,
    addr: &str,
    ip: &str,
) -> Result<(IpcStatus, Option<(String, String, String)>)> {
    let mut status = IpcStatus::Add;
    if global.toml_config.read().await.is_exist_ipc_config(uuid)? {
        let (user, password) = global.toml_config.read().await.get_ipc_auth(uuid)?;
        if _check_device_password(user.as_str(), password.as_str(), addr).is_ok() {
            return Ok((IpcStatus::Online, None));
            // } else {
            //     //TODO 删除本地数据，上层函数上报服务器
            //     global.toml_config.write().await   .remove_ipc_config(uuid)?;
            //     return Ok((IpcStatus::UpdateUnatuth, None));
        } else {
            status = IpcStatus::UpdateUnatuth;
        }
    }
    // 新的ipc
    if let Ok((user, password, manufacturer)) = _check_device_passwords(addr, global.clone()).await
    {
        if status == IpcStatus::UpdateUnatuth {
            status = IpcStatus::UpdateAuth;
        }
        // 获取基础数据，保存至配置，更新本地数据
        let url = format!("http://{}/onvif/media_service", ip.clone());
        let profile = get_profiles(user.as_str(), password.as_str(), &url)?;
        get_stream_url_and_init_config(
            uuid,
            user.as_str(),
            password.as_str(),
            manufacturer.as_str(),
            &profile,
            url.as_str(),
            global.clone(),
        )
        .await?;
        global.update_ipc_ips_tmp(uuid, ip).await;
        global.update_ipc_addr_tmp(uuid, addr).await;
        Ok((status, Some((user, password, manufacturer))))
    } else {
        warn!("ipc[{:?}]鉴权失败", uuid);
        //TODO 无法认证，上层函数上报服务器
        return Ok((IpcStatus::NewUnauth, None));
    }
}

async fn get_stream_url_and_init_config(
    uuid: &str,
    user: &str,
    password: &str,
    manufacturer: &str,
    profile: &Profile,
    url: &str,
    global: Arc<Global>,
) -> Result<()> {
    let stream_url = get_stream_url_xml(user, password, profile.name.as_str(), url)?;
    let mut lock = global.toml_config.write().await;
    lock.add_ipc_config(
        uuid,
        stream_url,
        user,
        password,
        manufacturer,
        profile.width,
        profile.height,
    )
    .await?;
    Ok(())
}

async fn _check_device_passwords(
    url: &str,
    global: Arc<Global>,
) -> Result<(String, String, String)> {
    let (users, passwords) = global.toml_config.read().await.get_ipc_auths()?;
    for user in &users {
        for password in &passwords {
            // debug!("{:?}-{:?}", user, password);
            if let Ok(info) = _check_device_password(user.as_str(), password.as_str(), url) {
                return Ok((user.clone(), password.clone(), info));
            }
        }
    }
    fail_from_str("匹配不到用户密码")
}

pub(crate) async fn check_device_password_by_cloud(
    user: &str,
    password: &str,
    uuid: &str,
    global: Arc<Global>,
) -> Result<()> {
    let lock = global.ipc_addr.read().await;
    let tmp = lock.get(uuid);
    let url = fail_from_option(tmp, "该ipc离线中...")?;
    let lock = global.ipc_ips.read().await;
    let tmp = lock.get(uuid);
    let ip = fail_from_option(tmp, "该ipc离线中...")?;
    if let Ok(info) = _check_device_password(user, password, url) {
        // 获取基础数据，保存至配置，更新本地数据
        let url = format!("http://{}/onvif/media_service", ip.clone());
        let profile = get_profiles(user, password, &url)?;
        get_stream_url_and_init_config(
            uuid,
            user,
            password,
            info.as_str(),
            &profile,
            url.as_str(),
            global.clone(),
        )
        .await?;
    }
    Ok(())
}
///
/// 验证用户密码、并获取厂商
fn _check_device_password(user: &str, password: &str, url: &str) -> Result<String> {
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(REQUEST02);
    head.push_str(GET_DEVICEINFORMATION);
    head.push_str(REQUEST04);
    // debug!("{:?}", head);
    let mut req = CallBuilder::post(Vec::from(head));
    let req = req
        .timeout_ms(5000)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .url(&url)?;
    for _ in [1..4].iter() {
        match req.exec() {
            Ok((_, body)) => {
                let res = String::from_utf8(body).unwrap();
                let doc = roxmltree::Document::parse(res.as_str()).unwrap();
                return _get_node_text("Manufacturer", &doc);
            }
            Err(_) => {
                warn!("请求ipc失败，重试...");
            }
        }
    }
    fail_from_str("请求ipc失败")
    // let (_, body) = CallBuilder::post(Vec::from(head))
    //     .timeout_ms(5000)
    //     .header("Content-Type", "application/soap+xml; charset=utf-8")
    //     .url(&url)
    //     .unwrap()
    //     .exec()?;
    // let res = String::from_utf8(body).unwrap();
    // let doc = roxmltree::Document::parse(res.as_str()).unwrap();
    // _get_node_text("Manufacturer", &doc)
    // if let Some(node) = doc
    //     .descendants()
    //     .find(|x| x.tag_name().name() == "Manufacturer")
    // {
    //     debug!("{:?}", node);
    //     return Ok(true);
    // }
    // fail_from_str("密码不匹配")
}

fn get_stream_url_xml(
    user: &str,
    password: &str,
    profile_token: &str,
    url: &str,
) -> Result<String> {
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(GET_STREAM_URL0);
    head.push_str(profile_token);
    head.push_str(GET_STREAM_URL1);
    let (_, body) = CallBuilder::post(Vec::from(head))
        .timeout_ms(5000)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .url(&url)
        .unwrap()
        .exec()?;
    let res = String::from_utf8(body).unwrap();
    // debug!("{:?}", res);
    let doc = roxmltree::Document::parse(res.as_str()).unwrap();

    _get_node_text("Uri", &doc)
}

fn get_profiles(user: &str, password: &str, url: &str) -> Result<Profile> {
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(GET_PROFILES);
    let (_, body) = CallBuilder::post(Vec::from(head))
        .timeout_ms(5000)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .url(&url)
        .unwrap()
        .exec()?;
    let res = String::from_utf8(body).unwrap();
    // debug!("{:?}", res);
    let doc = roxmltree::Document::parse(res.as_str()).unwrap();
    // _get_profiles(&doc)
    _get_profile(&doc)
}

fn get_auth_head(user: &str, password: &str) -> String {
    let mut head = AUTH_HEARD01.to_string();
    head.push_str(user);
    head.push_str(AUTH_HEARD02);
    let nonce = rand::random::<i32>().to_string();
    let time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    //2021-03-19T12:04:53.000Z
    // let time = "2021-03-19T12:04:53.000Z".to_string();
    let mut before = nonce.clone();
    before.push_str(time.as_str());
    before.push_str(password);
    // debug!("before=[{}]", before);
    let encrypt_password = base64::encode(Sha1::from(before).digest().bytes());
    // debug!(
    //     "[{}]-[{}]-[{}]-[{}]-[{}]",
    //     user, password, nonce, time, encrypt_password
    // );
    head.push_str(encrypt_password.as_str());
    head.push_str(AUTH_HEARD03);
    head.push_str(base64::encode(nonce).as_str());
    head.push_str(AUTH_HEARD04);
    head.push_str(time.as_str());
    head.push_str(AUTH_HEARD05);
    head
}

#[derive(Debug)]
struct Profile {
    pub name: String,
    pub width: u32,
    pub height: u32,
}
///寻找第一个Profile
fn _get_profile<'a>(doc: &'a Document) -> Result<Profile> {
    let nodes = doc
        .descendants()
        .find(|x| x.tag_name().name() == "Profiles");
    let node = fail_from_option(nodes, "找不到Profiles节点")?;
    _init_profile(node)
}
// 寻找所有的Profile。事实证明并不是所有的Profile都可以转成flv格式
fn _get_profiles<'a>(doc: &'a Document) -> Result<Vec<Profile>> {
    let nodes = doc
        .descendants()
        .filter(|x| x.tag_name().name() == "Profiles");
    let mut profiles: Vec<Profile> = Vec::with_capacity(10);
    for node in nodes {
        match _init_profile(node) {
            Ok(profile) => {
                profiles.push(profile);
            }
            Err(e) => {
                error!("{:?}", e);
                continue;
            }
        }
    }
    Ok(profiles)
}

fn _get_child_node<'a>(tag_name: &str, node: Node<'a, 'a>) -> Result<Node<'a, 'a>> {
    for children in node.children() {
        if children.tag_name().name() == tag_name {
            return Ok(children.clone());
        } else {
            if let Ok(child) = _get_child_node(tag_name, children) {
                return Ok(child);
            }
            continue;
        }
    }
    fail(format!("无法找到子节点:{}", tag_name))
}

fn _init_profile(node: Node) -> Result<Profile> {
    if let Some(name) = node.attribute("token") {
        // debug!("{:?}", node.children());
        if let Ok(node0) = _get_child_node("Width", node) {
            if let Some(text0) = node0.text() {
                if let Ok(node1) = _get_child_node("Height", node) {
                    if let Some(text1) = node1.text() {
                        return Ok(Profile {
                            name: name.to_string(),
                            width: text0.parse::<u32>()?,
                            height: text1.parse::<u32>()?,
                        });
                    }
                }
            }
        }
    }
    fail_from_str("获取profile失败：缺少必要数据")
}

// fn _get_nodes_attribute<'a>(tag_name: &str, attr: &str, doc: &'a Document) -> Result<Ver<&'a str>> {
//     if let Some(node) = doc.descendants().filter(|x| {
//         if x.tag_name().name() == tag_name {
//             return true;
//         }
//         return false;
//     }) {
//         if let Some(val) = node.attribute(attr) {
//             return Ok(val);
//         }
//     }
//     fail(format!("节点[{}]属性[{}]获取失败", tag_name, attr))
// }

fn _get_node_text<'a>(tag_name: &str, doc: &'a Document) -> Result<String> {
    if let Some(node) = doc.descendants().find(|x| x.tag_name().name() == tag_name) {
        if let Some(val) = node.text() {
            return Ok(val.to_string());
        }
    }
    fail(format!("节点[{}]文本获取失败", tag_name))
}

#[async_std::test]
async fn onvif_discovery_test() -> Result<()> {
    init_log();
    let config = global::init_config()?;
    // config.toml_config.read().await.get_ipcs();

    debug!("{:?}", onvif_discovery(config).await);
    Ok(())
}

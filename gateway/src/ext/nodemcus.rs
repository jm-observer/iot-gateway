#![allow(unused)]
use chrono::{DateTime, Local};
// use std::thread::sleep;
// use std::time::Duration;
use crate::*;
use async_std::net::UdpSocket;
use async_std::sync::RwLock;
use chrono::Duration;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/// mac地址、是否在线、判定为离线的时间
// struct McuStatus(String, bool, DateTime<Local>);

type NodeMcus = HashMap<String, DateTime<Local>>;

pub async fn nodemcu_task(global: Arc<Global>) -> Result<()> {
    // mac地址, (是否在线、判定为离线的时间)
    let mut nodemcus: NodeMcus = HashMap::with_capacity(10);

    let duration = Duration::minutes(1);
    let local = Local::now();
    let general_time = || Local::now() + duration.clone();

    let receiver: Receiver<VideoCommand> = global.nodemcu_command.1.clone();
    let udp = _init_udp_gateway().await?;
    let mut data = [0u8; 1024];
    let sleep_duration = std::time::Duration::from_secs(30);

    // udp.send_to("".to_string().as_bytes())
    loop {
        // let size = udp.recv(&mut data).await?;
        // // info!("size = {}", size);
        // debug!(
        //     "{:?}",
        //     deal_nodemcu_msg(&data[0..size], &mut nodemcus, general_time)
        // );
        select! {
            udp_data = udp.recv_from(&mut data).fuse() => match udp_data {
                Ok((size, from_addr)) => {
                    match deal_nodemcu_msg(&data[0..size], &mut nodemcus, general_time) {
                        Err(e) => error!("{:?}", e),
                        Ok(ack_msg) => {
                            debug!("{:?}", from_addr);
                            udp.send_to(ack_msg.as_bytes(), from_addr).await;
                        }
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            },
        min = async_std::task::sleep(sleep_duration).fuse() => {
            let now = Local::now();
            let mut offlines: Vec<String> = Vec::with_capacity(10);
            nodemcus = nodemcus
                .into_iter()
                .filter_map(|(key, item)| {
                    if item < now {
                        offlines.push(key);
                        None
                    } else {
                        Some((key, item))
                    }
                })
                .collect();
            debug!("acdfewefw{:?}", offlines);
        }
        command = receiver.recv().fuse() => {

        }
        }
    }
    Ok(())
}

use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Once;
use std::thread::sleep;

const START: Once = Once::new();
static mut OFFLINE: Option<Duration> = None;
static ONCE: Once = Once::new();

struct NodeMcuOrder<T> {
    uuid: *mut String,
    from: *mut String,
    to: *mut String,
    msg_type: *mut String,
    action: *mut String,
    msg: Json,
    phantom: PhantomData<T>,
}
impl<Heart> NodeMcuOrder<Heart> {
    fn action(&self, nodes: &mut NodeMcus, general: impl Fn() -> DateTime<Local>) {
        if let Some(node) = nodes.get_mut(self.from()) {
            *node = general();
        } else {
            nodes.insert(self.from().clone(), general());
            // todo 上线
        }
    }
}

fn deal_nodemcu_msg(
    data: &[u8],
    nodes: &mut NodeMcus,
    general: impl Fn() -> DateTime<Local>,
) -> Result<String> {
    let msg: Json = Json::parse(data)
        .map_err(|(pos, msg)| format!("`{}` at position `{}`!!!", msg, pos))
        .into_result()?;
    // debug!("msg = {:?}", msg);
    match msg.ext_get_ref_string("action") {
        Ok(action) => match action.as_str() {
            "heart" | "register" => {
                let order = NodeMcuOrder::<Heart>::new(msg)?;
                debug!("{:?}", order.from());
                order.action(nodes, general);
                Ok(order.ack())
            }
            _ => bail!(""),
        },
        Err(e) => Err(e),
    }
}
impl<'a, T> NodeMcuOrder<T> {
    fn new(msg: Json) -> Result<Self> {
        let uuid = std::ptr::null_mut();
        let from = std::ptr::null_mut();
        let msg_type = std::ptr::null_mut();
        let action = std::ptr::null_mut();
        let to = std::ptr::null_mut();
        let mut a = Self {
            uuid,
            from,
            to,
            msg_type,
            action,
            msg,
            phantom: PhantomData,
        };
        a.uuid = a.msg.ext_get_ref_string("uuid")? as *const String as *mut String;
        a.from = a.msg.ext_get_ref_string("from")? as *const String as *mut String;
        a.msg_type = a.msg.ext_get_ref_string("type")? as *const String as *mut String;
        a.action = a.msg.ext_get_ref_string("action")? as *const String as *mut String;
        a.to = a.msg.ext_get_ref_string("to")? as *const String as *mut String;
        Ok(a)
    }
    fn uuid(&self) -> &'a String {
        unsafe { &*self.uuid }
    }
    fn from(&self) -> &'a String {
        unsafe { &*self.from }
    }

    fn ack(self) -> String {
        unsafe {
            std::ptr::swap(self.from, self.to);
            *self.msg_type = String::from("ack");
        }
        self.msg.print()
    }
}

struct Heart;
struct Register;

async fn _init_udp_gateway() -> Result<UdpSocket> {
    let mut ip = get_ip()?;
    let local = ip.to_string();
    ip.push_str(":6000");
    debug!("{:?}", ip);
    let udp = UdpSocket::bind(ip).await?;
    udp.join_multicast_v4("224.0.211.211".parse()?, local.parse()?)?;
    Ok(udp)
}

#[test]
fn test() {
    init_log();
    let duration = Duration::minutes(1);
    let local = Local::now();
    let general_time = || Local::now() + duration;
    debug!("{:?}", general_time());
    sleep(std::time::Duration::from_secs(5));
    debug!("{:?}", general_time());
    sleep(std::time::Duration::from_secs(5));
    debug!("{:?}", general_time());
    if local < general_time() {
        debug!("{:?}", local);
    }
}
fn min() -> DateTime<Local> {
    Local::now()
}
static mut FN_STATIC: fn() -> DateTime<Local> = min;

#[test]
fn test_json() {
    init_log();
    let json = Json::STRING("abc".to_string());
    let pointer: *mut String = match json {
        Json::STRING(ref val) => val as *const String as *mut String,
        _ => std::ptr::null_mut(),
    };
    unsafe {
        *pointer = "cd".to_string();
    }
    println!("{:?}", json);
}

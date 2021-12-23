use std::fmt::Debug;

use crate::pub_use::*;
use ack_ipc::*;
use ele_collect::*;
use end_video::*;
use env_collect::*;
use heart_check::*;
pub use serial::*;
pub use ssh::*;

use gateway_common::*;

use super::event::upgrade::Upgrade;
use crate::ext_mqtt::event::req_images::ReqImages;
use crate::ext_mqtt::event::req_video::ReqVideo;

mod ack_ipc;
mod ele_collect;
mod end_video;
mod env_collect;
mod heart_check;
mod req_images;
mod req_ipc_image;
mod req_video;
mod serial;
mod ssh;
mod upgrade;

// impl<T> From<FailType> for Result<T> {
//     fn from(a: FailType) -> Self {
//         return Err(a);
//     }
// }

pub async fn init_event(msg: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
    let event = msg.get_event();
    match event {
        "ssh" => {
            //topic=>/28D24499AB4E_support/req/server/serverId/ssh
            //"{\"ip\":\"148.70.132.77\",\"ah\":\"localhost\",\"ap\":22,\"port\":20004,\"u\":\"2101281930273003\",\"order\":\"start\"}"
            //payload=>{"ip":"148.70.132.77", "port":20004, "ah":"dudusisi.asuscomm.com","ap":22000, "order":"start"}
            return Ssh::new(msg, global.clone()).await;
        }
        "heartCheck" => {
            return HeartCheck::new(msg, global.clone());
        }
        "upgrade" => {
            return Upgrade::new(msg, global.clone());
        }
        "reqVideo" => {
            return ReqVideo::new(msg, global.clone());
        }
        "endVideo" => {
            return EndVideo::new(msg, global.clone());
        }
        "ackIpc" => {
            return AckIpc::new(msg, global.clone());
        }
        "reqIpcImages" => {
            return ReqImages::new(msg, global.clone());
        }
        "eleCollect" => {
            return EleCollect::new(msg, global.clone());
        }
        "envCollect" => {
            return EnvCollect::new(msg, global.clone());
        }
        _ => {
            bail!("尚未实现事件（{}）!", event);
        }
    }
}

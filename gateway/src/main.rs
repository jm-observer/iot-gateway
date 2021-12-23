// extern crate iot_gateway;
use crate::pub_use::*;
use std::panic;
use std::time::Duration;

pub use async_std::channel::{bounded, Receiver, Sender};
pub use async_std::sync::Arc;
pub use async_std::sync::Mutex;
pub use async_std::task;
pub use async_trait::async_trait;
use futures::future::join4;
pub use futures::{future::join, pin_mut, select, FutureExt};
pub use json_minimal::Json;

use crate::cos::start_cos_client;
pub use crate::event::*;
pub use crate::ffmpeg::*;
use anyhow::Result;
pub use ext::*;
pub use ext_mqtt::*;
pub use gateway_common::*;
pub use mqtt::*;
pub use sub_task::*;

mod ext;
mod ext_mqtt;
mod ffmpeg;
mod mqtt;
mod pub_use;
mod sub_task;

// use std::result::Result;

#[cfg(target_family = "unix")]
pub fn init_log() {
    log4rs::init_file("./config/log4rs.yaml", Default::default()).unwrap();
    // log4rs::init_file("/home/pi/iot/config/log4rs‘.yaml", Default::default()).unwrap();
}
#[cfg(target_family = "windows")]
pub fn init_log() {
    log4rs::init_file("./config/log4rs.yaml", Default::default()).unwrap();
}

#[async_std::main]
async fn main() {
    if let Err(e) = main_detail().await {
        println!("main start fail: {:?}", e);
    };
}

async fn main_detail() -> Result<()> {
    /*
     * 设置log；读取配置信息；
     */
    let config = init_config()?;
    // 守护进程
    let config_clone = config.clone();
    let handler = task::spawn(async move {
        if let Err(e) = daemon().await {
            error!("{:?}", e);
        }
    });
    // ipc的onvif扫描任务
    let time = 60 * 30;
    let onvif_discovery = task::spawn(async move {
        loop {
            if let Err(err) = onvif_discovery(config_clone.clone()).await {
                error!("{:?}", err);
            }
            task::sleep(Duration::from_secs(time)).await;
        }
    });
    // 磁盘、内存、cpu检查任务
    let os_config = config.clone();
    let os_time = 60 * 30;
    task::spawn(async move {
        loop {
            if let Err(e) = disk_free(os_config.clone()).await {
                report_error_msg(os_config.clone(), &format!("磁盘检测出错：{:?}", e)).await;
            }
            if let Err(e) = memory_free(os_config.clone()).await {
                report_error_msg(os_config.clone(), &format!("内存检测出错：{:?}", e)).await;
            }
            if let Err(e) = cpu_usage(os_config.clone()).await {
                report_error_msg(os_config.clone(), &format!("cpu检测出错：{:?}", e)).await;
            }
            task::sleep(Duration::from_secs(os_time)).await;
        }
    });
    // let send_to_ffmpeg = config.sender_to_ffmpeg.clone();
    // ipc的图片、视频采集、转发任务
    let rec_video_command = config.rec_video_command.clone();
    let ffmpeg_config = config.clone();
    task::spawn(async move {
        if let Err(e) = ffmpeg_task(rec_video_command, ffmpeg_config).await {
            error!("{:?}", e);
        }
    });
    // nodemcu
    let nodemcu_config = config.clone();
    task::spawn(async move {
        if let Err(e) = nodemcu_task(nodemcu_config).await {
            error!("{:?}", e);
        }
    });
    // 内网ssh转发任务
    let rec_ssh = config.rec_ssh.clone();
    let ssh_config = config.clone();
    let ffmpeg = task::spawn(async move {
        if let Err(e) = ssh_task(rec_ssh, ssh_config).await {
            error!("{:?}", e);
        }
    });
    // 对象存储(cos)的上传任务
    start_cos_client(config.clone()).await?;
    // mqtt客户端
    let mqtt_config = config.clone();
    let mqtt = task::spawn(async move {
        if let Err(e) = mqtt::mqtt(mqtt_config).await {
            error!("{:?}", e);
        }
    });
    // 电流采集
    let ele_config = config.clone();
    task::spawn(async move { ele_collect_task(ele_config).await });
    // 环境信息采集
    let env_config = config.clone();
    task::spawn(async move { env_collect_task(env_config).await });
    // 配置参数同步
    let config_toml = config.clone();
    task::spawn(async move { update_config_task(config_toml).await });
    // 屏幕控制
    let screen_config = config.clone();
    task::spawn(async move { screen_control(screen_config).await });

    join4(onvif_discovery, handler, mqtt, ffmpeg).await;
    Ok(())
}

/// 相机图片、视频的相关实现
use crate::cos::CosTask;
use crate::*;
use async_std::task::JoinHandle;
use iot_gateway::VideoCommand::*;
use std::process::{Child, Command, ExitStatus, Stdio};

///
/// 参数ipc的uuid，time
///
pub async fn ffmpeg_task(reader: Receiver<VideoCommand>, global: Arc<Global>) -> Result<()> {
    // let (sender, receicer) = channel::unbounded::<VideoCommand>();
    let mut child: Option<Child> = None;
    let mut join_handle: Option<JoinHandle<()>> = None;
    while let Ok(command) = reader.recv().await {
        match command {
            VideoCommand::ReqVideo(packet) => {
                if is_exit_child(&mut child) {
                    child = None;
                } else {
                    let err = "已存在传输视频，无法再上传视频";
                    debug!("{}", err);
                    packet.ack_fail_packet(None, None, Some(err)).await;
                    continue;
                }
                let rtmp = match _get_rtmp_addr(global.clone()).await {
                    Ok(tmp) => tmp,
                    Err(e) => {
                        packet.ack_fail_from_failtype(&e).await;
                        continue;
                    }
                };
                match req_video(packet.clone(), &rtmp).await {
                    Ok(res) => {
                        let mut ext = Json::new();
                        ext.ext_add_str_object("url", &rtmp);
                        packet.ack_success_packet(Some(ext), None).await;
                        child = Some(res);
                    }
                    Err(e) => {
                        packet.ack_fail_from_failtype(&e).await;
                        continue;
                    }
                }
            }
            CloudEndVideo(packet) => {
                //云服务器人工停止命令，需要返回报文
                if let Some(ref mut child_tmp) = child {
                    if let Err(e) = child_tmp.kill() {
                        packet
                            .ack_fail_packet(None, None, Some(&format!("{:?}", e)))
                            .await;
                        continue;
                    }
                }
                child = None;
                packet.ack_brief_success_packet().await;
            }
            ReqImages(packet) => {
                let mut time = packet.get_msg().ext_get_int_or_default("time", 30f64);
                time = time * 60f64;
                //获取已鉴权的ipcs,拼装rtsp，启动定时任务：截图、上报cos，上报mqtt
                if time > 0f64 {
                    if join_handle.is_some() {
                        packet.ack_brief_success_packet().await;
                        continue;
                    }
                    match global.toml_config.read().await.get_ipcs_data() {
                        Ok(datas) => {
                            let packet_tmp = packet.clone();
                            let global_tmp = global.clone();
                            let handle = task::spawn(async move {
                                if let Err(e) = get_images(datas, time, global_tmp).await {
                                    packet_tmp.ack_fail_from_failtype(&e).await;
                                }
                            });
                            join_handle = Some(handle);
                            //先返回成功，后续再上传图片
                            packet.ack_brief_success_packet().await;
                        }
                        Err(e) => {
                            error!("{:?}", e);
                        }
                    };
                } else {
                    if let Some(handle) = join_handle {
                        handle.cancel().await;
                        debug!("程序取消成功");
                    }
                    join_handle = None;
                    // debug!("发送响应报文成功");
                    packet.ack_brief_success_packet().await;
                }
            }
        }
    }
    Ok(())
}

struct Ipc {
    uuid: String,
    user: String,
    password: String,
    pre_rtsp: String,
    // ip: Option<String>,
    // rtsp: Option<String>,
    // sub_uri: String,
    // images_path: String,
}
impl Ipc {
    async fn gen_image_param(
        &self,
        images_path: &str,
        sub_uri: &str,
        global: Arc<Global>,
    ) -> Result<(String, String, String, String)> {
        let ip_tmp = global.ipc_ips.read().await;
        if let Some(ip) = ip_tmp.get(self.uuid.as_str()) {
            let rtsp = format!(
                "rtsp://{}:{}@{}{}",
                self.user, self.password, ip, self.pre_rtsp
            );
            let image = uuid();
            let file_path = format!("{}/{}.png", images_path, image);
            let uri_path_name = format!("/{}/{}.png", sub_uri, image);
            return Ok((self.uuid.clone(), rtsp, file_path, uri_path_name));
        }
        fail_from_str("该ipc未在线！")
    }
}

async fn get_images(
    ipcs_tmp: Vec<(String, String, String, String, String)>,
    time: f64,
    global: Arc<Global>,
) -> Result<()> {
    let mut images_path = global
        .toml_config
        .read()
        .await
        .get_server_path("req_ipc_images", "/home/pi/images/");
    let today = get_today_string();
    images_path.push_str(&today);
    async_std::fs::create_dir_all(&images_path).await?;
    let images = global
        .toml_config
        .read()
        .await
        .get_two_level_string("cos-ext", "images")?;
    let sub_uri = format!("{}/{}/{}", &images, today, global.mqtt_id);
    let is_small = true;
    let ipcs = _get_ipcs_data(ipcs_tmp).await;
    loop {
        debug!("开始循环截图");
        // _update_ipcs_ip(&mut ipcs, global.clone()).await;
        // _update_ipcs_rtsp(&mut ipcs).await;
        for ipc in &ipcs {
            match ipc
                .gen_image_param(images_path.as_str(), sub_uri.as_str(), global.clone())
                .await
            {
                Ok((uuid, rtsp, file_path, uri_path_name)) => {
                    let global_tmp = global.clone();
                    task::spawn(async move {
                        if let Err(e) = get_ipc_image(
                            uuid,
                            rtsp,
                            file_path,
                            uri_path_name,
                            is_small,
                            global_tmp,
                        )
                        .await
                        {
                            error!("{:?}", e);
                        }
                    });
                }
                Err(e) => {
                    warn!("{:?}", e);
                }
            }
        }
        task::sleep(Duration::from_secs(time as u64)).await;
    }
}

pub async fn get_original_image(packet: Arc<MqttPacket>) -> Result<()> {
    let uuid = packet.get_msg().ext_get_string("uuid")?;
    let (uuid, name, password, rtsp, _) =
        packet.global.toml_config.read().await.get_ipc_data(&uuid)?;

    let mut images_path = packet
        .global
        .toml_config
        .read()
        .await
        .get_server_path("req_ipc_images", "/home/pi/images/");
    let today = get_today_string();
    images_path.push_str(&today);
    async_std::fs::create_dir_all(&images_path).await?;
    let images = packet
        .global
        .toml_config
        .read()
        .await
        .get_two_level_string("cos-ext", "images")?;
    let sub_uri = format!("{}/{}/{}", &images, today, packet.global.mqtt_id);
    let is_small = false;

    let ipc = Ipc {
        uuid,
        user: name,
        password,
        pre_rtsp: rtsp.to_string(),
    };
    let (uuid, rtsp, file_path, uri_path_name) = ipc
        .gen_image_param(
            images_path.as_str(),
            sub_uri.as_str(),
            packet.global.clone(),
        )
        .await?;
    let global_tmp = packet.global.clone();
    if let Err(e) = get_ipc_image(uuid, rtsp, file_path, uri_path_name, is_small, global_tmp).await
    {
        error!("{:?}", e);
    }
    Ok(())
}

async fn get_ipc_image(
    uuid: String,
    rstp: String,
    file_path: String,
    uri_path_name: String,
    is_small: bool,
    global: Arc<Global>,
) -> Result<()> {
    // let image = uuid();
    // if let Some(ref rstp) = ipc.rtsp {
    //     let file_path = format!("{}/{}.png", ipc.images_path, image);
    //     let uri_path_name = format!("/{}/{}.png", ipc.sub_uri, image);
    debug!("开始截图");
    let child = if is_small {
        init_small_png_child(rstp.as_str(), &file_path)?
    } else {
        init_original_png_child(rstp.as_str(), &file_path)?
    };
    debug!("完成截图");
    if child.success() {
        let (sender_res, rec_res) = bounded::<Result<String>>(1024);
        let ct = CosTask {
            is_wait: true,
            file_path,
            file_uri_path: uri_path_name,
            send_res: sender_res,
        };
        //开始上报cos
        global.sender_to_cos.send(ct).await?;
        if let Ok(Ok(url)) = rec_res.recv().await {
            let mut msg = Json::new();
            msg.ext_add_str_object("uuid", uuid.as_str())
                .ext_add_str_object("image", &url);
            send_req_to_mqtt(global.clone(), "reqIpcImage", msg, None).await;
        } else {
            report_error_msg(global, "图片上传cos失败！").await;
        }
        //上报cos结束
    }
    // }
    Ok(())
}
async fn _get_ipcs_data(ipcs: Vec<(String, String, String, String, String)>) -> Vec<Ipc> {
    let mut rstps = Vec::<Ipc>::with_capacity(20);
    // let ip_tmp = packet.global.ipc_ips.read().await;
    for (uuid, name, password, rtsp, _) in ipcs {
        rstps.push(Ipc {
            uuid,
            user: name,
            password,
            pre_rtsp: rtsp.to_string(),
        });
    }
    rstps
}

fn is_exit_child(child: &mut Option<Child>) -> bool {
    if let Some(ref mut ch) = child {
        match ch.try_wait() {
            Ok(Some(status)) => {
                //已结束
                debug!("上次传输视频的结果：{:?}", status);
                return true;
            }
            Ok(None) => {
                //未结束
                return false;
            }
            Err(e) => {
                //异常
                error!("传输视频异常：{:?}", e);
                return true;
            }
        }
    } else {
        return true;
    }
}

async fn _get_rtmp_addr(global: Arc<Global>) -> Result<String> {
    let config = global.toml_config.read().await;
    let rtmp = config.get_server_config_string("rtmp")?;
    let uuid_pull = uuid();
    Ok(format!("rtmp://{}{}", rtmp, uuid_pull))
}

async fn req_video(packet: Arc<MqttPacket>, rtmp: &str) -> Result<Child> {
    let config = packet.global.toml_config.read().await;
    let uuid_tmp = packet.get_msg().ext_get_string("uuid")?;
    let rtsp = config.get_ipc_rtsp(&uuid_tmp)?;
    let (user, password) = config.get_ipc_auth(&uuid_tmp)?;
    let ip_tmp = packet.global.ipc_ips.read().await;
    debug!("req_video={:?}", ip_tmp);
    let ip = fail_from_option(ip_tmp.get(&uuid_tmp), "该ipc离线中！")?;
    let time = packet.get_msg().ext_get_int_or_default("time", 60f64) as u64;

    let may_panic = async {
        init_child(
            &format!("rtsp://{}:{}@{}{}", user, password, ip, rtsp),
            rtmp,
            &time.to_string(),
        )
    };
    match may_panic.catch_unwind().await {
        Err(e) => {
            let msg = format!("视频传输异常，可能是未安装ffmpeg：{:?}", e);
            return fail(msg);
        }
        Ok(child) => return child,
    };
}

fn init_child(rtsp: &str, rtmp: &str, time: &str) -> Result<Child> {
    fail_from_result(
        Command::new("ffmpeg")
            .arg("-r")
            .arg("25")
            .arg("-t")
            .arg(time)
            .arg("-i")
            .arg(rtsp)
            .arg("-vcodec")
            .arg("libx264")
            .arg("-tune")
            .arg("zerolatency")
            .arg("-preset")
            .arg("ultrafast")
            .arg("-b:v")
            .arg("400k")
            .arg("-f")
            .arg("flv")
            .arg("-tune")
            .arg("zerolatency")
            .arg(rtmp)
            .stdout(Stdio::null())
            .spawn(),
    )
}

fn init_small_png_child(rtsp: &str, local_path: &str) -> Result<ExitStatus> {
    fail_from_result(
        Command::new("ffmpeg")
            .arg("-i")
            .arg(rtsp)
            .arg("-y")
            .arg("-vf")
            .arg("scale=320:240")
            .arg("-vframes")
            .arg("1")
            .arg(local_path)
            .spawn()?
            .wait(),
    )
}
fn init_original_png_child(rtsp: &str, local_path: &str) -> Result<ExitStatus> {
    fail_from_result(
        Command::new("ffmpeg")
            .arg("-i")
            .arg(rtsp)
            .arg("-y")
            .arg("-vframes")
            .arg("1")
            .arg(local_path)
            .spawn()?
            .wait(),
    )
}

/// 内网转发ssh连接的相关操作
use crate::pub_use::*;
use async_std::io::copy;
use async_std::net::TcpStream;
use async_std::task::JoinHandle;
use futures::{pin_mut, select, FutureExt};
use crate::{MqttPacket, Global};

#[derive(Debug)]
pub struct TargetSsh {
    pub is_start: bool,
    pub ip: String,
    pub port: f64,
    pub ah: String,
    pub ap: f64,
    pub packet: Arc<MqttPacket>,
}

impl Clone for TargetSsh {
    fn clone(&self) -> Self {
        TargetSsh {
            is_start: self.is_start,
            ip: self.ip.clone(),
            port: self.port,
            ah: self.ah.clone(),
            ap: self.ap,
            packet: self.packet.clone(),
        }
    }
}

#[allow(unused_variables)]
pub async fn ssh_task(reader: Receiver<TargetSsh>, global: Arc<Global>) -> Result<()> {
    let mut jh: Option<JoinHandle<()>> = None;
    loop {
        //先简单处理就好了：有新的连接请求，则直接杀掉旧连接
        match reader.recv().await {
            Ok(ts) => {
                if ts.is_start {
                    if let Some(tmp0) = jh {
                        tmp0.cancel().await;
                    }
                    let ip = ts.ip;
                    let port = ts.port;
                    let ah = ts.ah;
                    let ap = ts.ap;
                    jh = Some(task::spawn(async move {
                        if let Err(e) = start(ip, port, ap, ah).await {
                            warn!("{:?}", e);
                        }
                    }));
                } else {
                    if let Some(tmp) = jh {
                        tmp.cancel().await;
                    }
                    jh = None;
                }
                ts.packet.ack_brief_success_packet().await;
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }
}

pub async fn start(ip: String, port: f64, ap: f64, ah: String) -> Result<()> {
    debug!("开始ssh连接: {}:{}...", ip, port);
    if let Ok(cloud_stream) = TcpStream::connect(format!("{}:{}", ip, port)).await {
        debug!("开始ssh连接：{}:{} ...", ah, ap);
        if let Ok(local_stream) = TcpStream::connect(format!("{}:{}", ah, ap)).await {
            debug!("开始ssh连接：数据开始交换");
            let (cloud_reader, cloud_writer) = &mut (&cloud_stream, &cloud_stream);
            let (local_reader, local_writer) = &mut (&local_stream, &local_stream);
            let res0 = copy(cloud_reader, local_writer).fuse();
            let res1 = copy(local_reader, cloud_writer).fuse();

            pin_mut!(res0, res1);
            select! {
                _ = res0 => {
                    bail!("云数据转发至本地中断");
                },
                _ = res1 => {
                    bail!("本地数据转发至云中断");
                },
            };
        } else {
            bail!("{}:{}连接失败", ah, ap);
        }
    } else {
        bail!("{}:{}连接失败", port, ip);
    }
}

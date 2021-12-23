use super::*;

#[derive(Debug, SuperActionImpl)]
pub struct Ssh {
    ts: TargetSsh,
    pub packet: Arc<MqttPacket>,
    pub global: Arc<Global>,
}

unsafe impl Send for Ssh {}
unsafe impl Sync for Ssh {}

impl Ssh {
    pub async fn new(packet: MqttPacket, global: Arc<Global>) -> Result<Arc<dyn Action>> {
        let msg = packet.get_msg();
        let ip = msg.ext_get_string("ip")?;
        let port = msg.ext_get_int("port")?;
        // let (sender_to_ssh, rec_from_main_0) = bounded::<bool>(1024);
        let ap = msg.ext_get_int_or_default("ap", 22f64);
        let ah = msg.ext_get_str_or_default("ah", "localhost");
        let is_start = match msg.ext_get_str_or_default("order", "end").as_str() {
            "start" => true,
            _ => false,
        };
        let arc_packet = Arc::new(packet);
        let ts = TargetSsh {
            is_start,
            ip,
            port,
            ap,
            ah,
            packet: arc_packet.clone(),
        };
        let ssh_client = Ssh {
            ts,
            packet: arc_packet.clone(),
            global: global.clone(),
        };

        // let mut lock = global.ssh_lock.lock().await;
        // if ssh_client.is_start() {
        //     if let Some(pre_ssh) = &lock.ssh {
        //         debug!("主程序重新启动ssh");
        //         pre_ssh.stop().await;
        //     }
        // }
        let box_ssh = Arc::new(ssh_client);
        // lock.ssh = Some(box_ssh.clone());
        Ok(box_ssh)
    }

    // pub fn is_start(&self) -> bool {
    //     match self.order {
    //         SshOrder::Start => true,
    //         _ => false,
    //     }
    // }
    //
    // pub async fn stop(&self) {
    //     if let Err(e) = self.sender_to_ssh.send(true).await {
    //         error!("主程序停止ssh失败：{:?}", e);
    //     }
    //     return ();
    // }

    // pub async fn start(&self) -> Result<()> {
    //     debug!("开始ssh连接...");
    //     if let Ok(cloud_stream) = TcpStream::connect(format!("{}:{}", self.ip, self.port)).await {
    //         if let Ok(local_stream) = TcpStream::connect(format!("{}:{}", self.ah, self.ap)).await {
    //             debug!("连接成功");
    //             self.packet.ack_success_packet(None, None).await;
    //             // self.send_success_ack(None, None).await?;
    //             let (cloud_reader, cloud_writer) = &mut (&cloud_stream, &cloud_stream);
    //             let (local_reader, local_writer) = &mut (&local_stream, &local_stream);
    //             let res0 = copy(cloud_reader, local_writer).fuse();
    //             let res1 = copy(local_reader, cloud_writer).fuse();
    //             let res2 = self.receiver_from_main.recv().fuse();
    //             pin_mut!(res0, res1, res2);
    //             // Fuse::new();
    //             select! {
    //                 _ = res0 => {
    //                     return fail("云数据转发至本地中断".to_string());
    //                 },
    //                 _ = res1 => {
    //                     return fail("本地数据转发至云中断".to_string());
    //                 },
    //                 _ = res2 => {
    //                     return Ok(());
    //                 }
    //             };
    //         } else {
    //             self.packet
    //                 .ack_fail_packet(
    //                     None,
    //                     None,
    //                     Some(format!("{}:{}连接失败", self.ah, self.ap).as_str()),
    //                 )
    //                 .await;
    //             // self.send_fail_ack()
    //             //     .await
    //         }
    //     } else {
    //         self.packet
    //             .ack_fail_packet(
    //                 None,
    //                 None,
    //                 Some(format!("{}:{}连接失败", self.ah, self.ap).as_str()),
    //             )
    //             .await;
    //         // self.send_fail_ack(format!("{}:{}连接失败", self.ip, self.port))
    //         //     .await
    //     }
    //     Ok(())
    // }
}
#[async_trait]
impl Action for Ssh {
    #[allow(unused_variables)]
    async fn detail_action(&self) -> Result<()> {
        if let Err(e) = self.global.sender_to_ssh.send(self.ts.clone()).await {
            self.packet
                .ack_fail_from_failtype(&FailTypeEnum::Fail("ssh连接失败".to_string()))
                .await;
        }
        Ok(())
    }
}
#[async_trait]
impl AckAction for Ssh {
    async fn ack_action(&self) {}
}

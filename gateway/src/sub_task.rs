use crate::pub_use::*;
use futures::StreamExt;
use heim::cpu::time;
use heim::units::information;
use heim::{disk, memory, units};
use std::ops::DerefMut;
use std::time::Duration;

/// 相关子任务
use crate::*;

const HOUR: u64 = 60u64 * 60;
const MINUTER: u64 = 60u64;
/// 环境（温度、湿度、气压）采集任务
pub async fn env_collect_task(global: Arc<Global>) {
    let mut frequency = 1 as u64 * 60;
    let fail_time = 60u64 * 60 * 12;
    let inavtive_time = 60u64 * 60 * 24;
    let mut inavtive: u64 = inavtive_time;
    let mut fail: u64 = fail_time;
    let no = global.get_two_level_string("envCollect", "no").await;
    loop {
        if let Ok(env_frequency) = global.get_two_level_int("envCollect", "frequency").await {
            frequency = env_frequency as u64 * 60;
        } else {
            task::sleep(Duration::from_secs(HOUR)).await;
        }
        match env_serial_task_detail(global.clone()).await {
            Ok((air, humidity, temp)) => {
                if no.is_err() {
                    report_error_msg(global.clone(), "未配置环境传感器的设备编号").await;
                    task::sleep(Duration::from_secs(HOUR)).await;
                    continue;
                }
                //上报
                let mut msg = Json::new();
                msg.ext_add_f64_object("airp", get_format_f64(air))
                    .ext_add_f64_object("humidity", get_format_f64(humidity))
                    .ext_add_f64_object("temp", get_format_f64(temp))
                    .ext_add_str_object("devNo", no.as_ref().unwrap().as_str());
                global
                    .send_to_mqtt(MqttPacket::new_req_packet(
                        "envCollect",
                        msg,
                        global.clone(),
                        None,
                    ))
                    .await;
            }
            Err(err) => {
                //未启用，则无其他操作
                // warn!("{}", err);
                if inavtive > inavtive_time {
                    report_info_msg(global.clone(), err.to_string().as_str()).await;
                    inavtive = 0u64;
                } else {
                    inavtive = inavtive + frequency;
                }
            }
            Err(err) => {
                // warn!("{}", err);
                if fail > fail_time {
                    report_error_msg(global.clone(), err.to_string().as_str()).await;
                    fail = 0u64;
                } else {
                    fail = fail + frequency;
                }
                task::sleep(Duration::from_secs(MINUTER)).await;
                continue;
            }
        }
        task::sleep(Duration::from_secs(frequency)).await;
    }
}
/// 电流大小采集任务
pub async fn ele_collect_task(global: Arc<Global>) {
    let mut frequency = 5u64 * 60 as u64;
    let fail_time = 60u64 * 60 * 12;
    let inavtive_time = 60u64 * 60 * 24;
    let mut inavtive: u64 = inavtive_time;
    let mut fail: u64 = fail_time;
    let no = global.get_two_level_string("eleCollect", "no").await;
    loop {
        if let Ok(env_frequency) = global.get_two_level_int("eleCollect", "frequency").await {
            frequency = env_frequency as u64 * 60;
        } else {
            task::sleep(Duration::from_secs(HOUR)).await;
        }
        match ele_serial_task_detail(global.clone()).await {
            Ok((va, vb, vc, vd)) => {
                if no.is_err() {
                    report_error_msg(global.clone(), "未配置电流传感器的设备编号").await;
                    task::sleep(Duration::from_secs(HOUR)).await;
                    continue;
                }
                //上报
                let mut msg = Json::new();
                msg.ext_add_f64_object("va", get_format_f64(va))
                    .ext_add_f64_object("vb", get_format_f64(vb))
                    .ext_add_f64_object("vc", get_format_f64(vc))
                    .ext_add_f64_object("vd", get_format_f64(vd))
                    .ext_add_str_object("devNo", no.as_ref().unwrap().as_str());
                global
                    .send_to_mqtt(MqttPacket::new_req_packet(
                        "eleCollect",
                        msg,
                        global.clone(),
                        None,
                    ))
                    .await;
            }
            Err(err) => {
                //未启用，则无其他操作
                // warn!("{}", err);
                if inavtive > inavtive_time {
                    report_info_msg(global.clone(), err.to_string().as_str()).await;
                    inavtive = 0u64;
                } else {
                    inavtive = inavtive + frequency;
                }
            }
            Err(err) => {
                // warn!("{}", err);
                if fail > fail_time {
                    report_error_msg(global.clone(), err.to_string().as_str()).await;
                    fail = 0u64;
                } else {
                    fail = fail + frequency;
                }
                task::sleep(Duration::from_secs(MINUTER)).await;
                continue;
            }
        }
        task::sleep(Duration::from_secs(frequency)).await;
    }
}

/// 磁盘大小
pub async fn disk_free(global: Arc<Global>) -> Result<()> {
    let partitions = disk::partitions_physical().await?;
    futures::pin_mut!(partitions);
    let deadline = global.toml_config.read().await.get_os_config_f64("disk")?;
    while let Some(part) = partitions.next().await {
        let part = part?;
        let usage = part.usage().await?;
        // debug!(
        //     "{:<17} {:<10} {:<10} {:<10} {:<10} {}",
        //     part.device()
        //         .unwrap_or_else(|| OsStr::new("N/A"))
        //         .to_string_lossy(),
        //     usage.total().get::<information::megabyte>(),
        //     usage.used().get::<information::megabyte>(),
        //     usage.free().get::<information::megabyte>(),
        //     part.file_system().as_str(),
        //     part.mount_point().to_string_lossy(),
        // );
        let rate = usage.used().get::<information::megabyte>() as f64
            / usage.total().get::<information::megabyte>() as f64;
        if rate > deadline {
            report_warn_msg(
                global.clone(),
                &format!(
                    "磁盘[{:}]占用率{:.2}%超过预警值[{:.2}%]",
                    part.mount_point().to_string_lossy(),
                    rate * 100f64,
                    deadline * 100f64
                ),
            )
            .await;
        }
    }
    Ok(())
}
/// 内存检查任务
pub async fn memory_free(global: Arc<Global>) -> Result<()> {
    let memory = memory::memory().await?;
    let swap = memory::swap().await?;

    let memory_deadline = global
        .toml_config
        .read()
        .await
        .get_os_config_f64("memory")?;
    let swap_deadline = global.toml_config.read().await.get_os_config_f64("swap")?;

    // debug!("              total        free   available");

    let memory_rate = memory.available().get::<information::megabyte>() as f64
        / memory.total().get::<information::megabyte>() as f64;

    if memory_rate > memory_deadline {
        report_warn_msg(
            global.clone(),
            &format!(
                "内存占用率{:.2}%超过预警值[{:.2}%]",
                memory_rate * 100f64,
                memory_deadline * 100f64
            ),
        )
        .await;
    }

    // debug!(
    //     "{:>7} {:>11?} {:>11?} {:>11?} {:>11?}",
    //     "Mem:",
    //     memory.total().get::<information::megabyte>(),
    //     memory.free().get::<information::megabyte>(),
    //     memory.available().get::<information::megabyte>(),
    //     memory_rate,
    // );
    let swap_rate = swap.used().get::<information::megabyte>() as f64
        / swap.total().get::<information::megabyte>() as f64;

    if swap_rate > swap_deadline {
        report_warn_msg(
            global.clone(),
            &format!(
                "swap占用率{:.2}%超过预警值[{:.2}%]",
                swap_rate * 100f64,
                swap_deadline * 100f64
            ),
        )
        .await;
    }
    // debug!(
    //     "{:>7} {:>11?} {:>11?} {:>11?} {:>11?}",
    //     "Swap:",
    //     swap.total().get::<information::megabyte>(),
    //     swap.used().get::<information::megabyte>(),
    //     swap.free().get::<information::megabyte>(),
    //     swap_rate,
    // );

    Ok(())
}
/// cpu使用检查任务
pub async fn cpu_usage(global: Arc<Global>) -> Result<()> {
    let cpu_time0 = time().await.unwrap();
    async_std::task::sleep(Duration::from_secs(2)).await;
    let cpu_time1 = time().await.unwrap();
    let delta_proc =
        (cpu_time1.user() - cpu_time0.user()) + (cpu_time1.system() - cpu_time0.system());
    let ide_time = cpu_time1.idle() - cpu_time0.idle();
    let all_time = delta_proc + ide_time;
    let overall_cpus_ratio =
        delta_proc.get::<units::time::second>() / all_time.get::<units::time::second>();

    let cpu_deadline = global.toml_config.read().await.get_os_config_f64("cpu")?;

    if overall_cpus_ratio > cpu_deadline {
        report_warn_msg(
            global.clone(),
            &format!(
                "cpu占用率{:.2}%超过预警值[{:.2}%]",
                overall_cpus_ratio * 100f64,
                cpu_deadline * 100f64
            ),
        )
        .await;
    }
    // debug!("{:?}", overall_cpus_ratio);
    Ok(())
}
/// 同步配置文件任务
pub async fn update_config_task(global: Arc<Global>) -> Result<()> {
    let refresh = 60u64 * 5;
    loop {
        task::sleep(Duration::from_secs(refresh)).await;
        let mut config = global.toml_config.write().await;
        let tmp = config.deref_mut();
        if let Err(_) = tmp.update_doc() {
            error!("config.toml 更新失败");
        }
    }
}
/// 屏幕控制任务
pub async fn screen_control(global: Arc<Global>) {
    let refresh = 60u64 * 5;
    loop {
        if let Ok(setting) = global
            .toml_config
            .read()
            .await
            .get_two_level_string("server", "screen")
        {
            if "on" == setting {
                match _screen_control_detail(global.clone()).await {
                    Ok(_) => {}
                    Err(fail) => {
                        //todo 应该上报？
                        warn!("{:?}", fail);
                    }
                }
            }
        }
        task::sleep(Duration::from_secs(refresh)).await;
    }
}

async fn _screen_control_detail(global: Arc<Global>) -> Result<()> {
    let screen_on_time = global
        .toml_config
        .read()
        .await
        .get_two_level_string("server", "screen_on")?;
    let screen_off_time = global
        .toml_config
        .read()
        .await
        .get_two_level_string("server", "screen_off")?;

    if screen_on_time > screen_off_time {
        return bail!(
            "屏幕控制时间错误：screen_on_time[{}]> screen_off_time[{}]",
            screen_on_time,
            screen_off_time
        );
    }
    let now = get_hour_minuter_string();
    if now < screen_on_time || now > screen_off_time {
        //关闭屏幕 xset -display :0.0 dpms force off
        if let Ok(status) = turn_off_screen() {
            if status.success() {
                return Ok(());
            }
        }
    } else if now >= screen_on_time && now <= screen_off_time {
        //打开屏幕
        if let Ok(status) = turn_on_screen() {
            if status.success() {
                return Ok(());
            }
        }
    }
    Ok(())
}

#[async_std::test()]
async fn test_os() {
    use crate::global::init_config;

    let global = init_config().unwrap();
    disk_free(global.clone()).await.unwrap();
    memory_free(global.clone()).await.unwrap();
    cpu_usage(global.clone()).await.unwrap();
}
/// 守护进程
#[cfg(target_family = "windows")]
pub async fn daemon() -> Result<()> {
    Ok(())
}
/// 守护进程
// #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
#[cfg(target_family = "unix")]
pub async fn daemon() -> Result<()> {
    use libsystemd::daemon::{self, NotifyState};

    if !daemon::booted() {
        error!("Not running systemd, early exit.");
        return Ok(());
    };

    let timeout = match daemon::watchdog_enabled(true) {
        Some(time) => time,
        None => return fail_from_str("watchdog_enabled None"),
    };
    debug!("daemon timeout = {}s", timeout.as_secs());
    loop {
        match daemon::notify(false, &[NotifyState::Watchdog]) {
            Ok(_res) => {
                debug!("deamon::notify true");
            }
            Err(err) => {
                error!("daemon error: {}", err);
                break;
            }
        }
        task::sleep(Duration::from_secs(60u64)).await;
    }
    Ok(())
}

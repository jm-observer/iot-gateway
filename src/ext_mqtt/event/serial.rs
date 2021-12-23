//! Simple example that echoes received serial traffic to stdout
use crate::*;
use std::str;

#[cfg(target_family = "unix")]
const INTERS: [&str; 2] = ["/dev/ttyUSB0", "/dev/ttyUSB1"];
#[cfg(target_family = "windows")]
const INTERS: [&str; 2] = ["COM3", "COM4"];

pub async fn ele_serial_task_detail(global: Arc<Global>) -> Result<(f64, f64, f64, f64)> {
    let ele_status = global.get_two_level_string("eleCollect", "status").await?;
    if ele_status == "on" {
        let inter = global
            .toml_config
            .read()
            .await
            .get_two_level_string("eleCollect", "inter")
            .unwrap_or(INTERS[0].to_string());
        let res = ele_info(inter.as_str()).await;
        if res.is_ok() {
            return res;
        } else {
            debug!(
                "电流检测原配置接口检测失败: {:?}，重新筛选接口...",
                res.unwrap_err()
            );
            for inter in INTERS.iter() {
                let res = ele_info(inter).await;
                if res.is_ok() {
                    global
                        .update_two_level_string("eleCollect", "inter", inter)
                        .await;
                    return res;
                }
            }
        }
        return fail_from_str("电流数据各接口均读取失败。");
    } else {
        return fail_from_str("电流检测未启用。");
    }
}

pub async fn env_serial_task_detail(global: Arc<Global>) -> Result<(f64, f64, f64)> {
    let env_status = global.get_two_level_string("envCollect", "status").await?;
    if env_status == "on" {
        let inter = global
            .toml_config
            .read()
            .await
            .get_two_level_string("envCollect", "inter")
            .unwrap_or(INTERS[0].to_string());
        let res = env_info(inter.as_str()).await;
        if res.is_ok() {
            return res;
        } else {
            debug!(
                "环境检测原配置接口检测失败: {:?}，重新筛选接口...",
                res.unwrap_err()
            );
            for inter in INTERS.iter() {
                let res = env_info(inter).await;
                if res.is_ok() {
                    global
                        .update_two_level_string("envCollect", "inter", inter)
                        .await;
                    return res;
                }
            }
        }
        return fail_from_str("温湿气压数据各接口均读取失败。");
    } else {
        return fail_from_str("环境检测未启用。");
    }
}

const AIR_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x0B, 0x00, 0x01, 0xF5, 0xC8];
// const AIR_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x34, 0x45, 0xDE];

const TEMP_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x02, 0xC4, 0x0B];
const ELE_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x04, 0xC5, 0xCB];
// const ELE_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x34, 0x45, 0xDE];

async fn ele_info(interface: &str) -> crate::Result<(f64, f64, f64, f64)> {
    let (mut rx, poll) = get_serial(interface, 50).await?;
    let res = get_serial_val(&mut rx, &poll, &ELE_INPUT)?;
    drop(rx);
    drop(poll);
    let mut tmp = [0u8; 2];
    tmp.copy_from_slice(&res[3..5]);
    let a = u16::from_be_bytes(tmp) as f64 * 0.01;
    tmp.copy_from_slice(&res[5..7]);
    let b = u16::from_be_bytes(tmp) as f64 * 0.01;
    tmp.copy_from_slice(&res[7..9]);
    let c = u16::from_be_bytes(tmp) as f64 * 0.01;
    //读到温湿气压传感器的值：{"va":655.35,"vb":655.35,"vc":655.35,"vd":101.93,"u":"a58186c2-4b4a-486c-90f2-b94a5f704531"}
    if a > 600f64 && b > 600f64 && c > 600f64 {
        return fail_from_str("接口读错：疑似读取温湿气压传感器");
    }
    tmp.copy_from_slice(&res[9..11]);
    let d = u16::from_be_bytes(tmp) as f64 * 0.01;
    Ok((a, b, c, d))
}

async fn env_info(interface: &str) -> crate::Result<(f64, f64, f64)> {
    let (mut rx, poll) = get_serial(interface, 150).await?;
    let res = get_serial_val(&mut rx, &poll, &AIR_INPUT)?;
    let mut tmp = [0u8; 2];
    tmp.copy_from_slice(&res[3..5]);
    let air = u16::from_be_bytes(tmp) as f64 * 0.1;
    //读取电流传感器的数值也许是：{"airp":0,"humidity":0.65,"temp":4,"u":"1f10a416-8679-48ea-b245-611196b5cf87"}
    if air < 300f64 {
        return fail_from_str("接口读错：疑似读取电流传感器");
    }
    let res = get_serial_val(&mut rx, &poll, &TEMP_INPUT)?;
    drop(rx);
    drop(poll);
    tmp.copy_from_slice(&res[3..5]);
    let humidity = u16::from_be_bytes(tmp) as f64 * 0.001;
    tmp.copy_from_slice(&res[5..7]);
    let temp = u16::from_be_bytes(tmp) as f64 * 0.1;
    Ok((air, humidity, temp))
}

#[test]
fn test_env() {
    init_log();
    // print!("{:?}", env_info(InterfaceType::Usb));
    // print!("{:?}", ele_info("/dev/ttyUSB0")).await;
}

use std::fmt;
use std::fmt::Debug;
use std::net::IpAddr;
use std::num::ParseIntError;
use std::process::{Command, ExitStatus};
use std::string::FromUtf8Error;

use async_std::net::AddrParseError;
use backtrace::{Backtrace, BacktraceFmt, BytesOrWideString, PrintFmt};
use chrono::{Local, NaiveDateTime};
use mac_address::MacAddressError;
use regex::Captures;
use rumqttc::ClientError;
use toml_edit::TomlError;
use uuid::Uuid;

use crate::*;
pub use const_str::*;
pub use ext_cos::*;
pub use ext_json::*;
pub use ext_json::*;
pub use global::*;
pub use packet::*;
pub use serial::*;
pub use ssh::*;
pub use toml_config::*;

mod const_str;
pub mod cos;
mod ext_cos;
mod ext_json;
pub mod global;
mod packet;
mod serial;
mod ssh;
mod toml_config;

pub type Result<T> = std::result::Result<T, FailTypeEnum>;

// pub fn get_mac_address() -> Result<String> {
//     use mac_address::get_mac_address;
//     if let Ok(Some(val)) = get_mac_address() {
//         return Ok(val.to_string().replace(":", ""));
//     }
//     fail("无法获取mac地址！".to_string())
// }

#[cfg(target_family = "windows")]
pub fn get_mac_address() -> Result<String> {
    let mut addres: String = String::from("");
    let mut is_none = false;
    if let Some(addr) = mac_address::mac_address_by_name("以太网")? {
        addres.push_str(addr.to_string().replace(":", "").as_str());
    } else {
        is_none = true;
    }
    if let Ok(Some(addr)) = mac_address::mac_address_by_name("WLAN") {
        addres.push_str("-");
        addres.push_str(addr.to_string().replace(":", "").as_str());
    } else if is_none {
        return fail_from_str("无法获取（windows有线无线）mac地址！");
    }
    Ok(addres)
}
#[cfg(target_family = "unix")]
pub fn get_mac_address() -> Result<String> {
    let mut addres: String = String::from("");
    let mut is_none = false;
    if let Some(addr) = mac_address::mac_address_by_name("eth0")? {
        addres.push_str(addr.to_string().replace(":", "").as_str());
    } else {
        is_none = true;
    }
    if let Ok(Some(addr)) = mac_address::mac_address_by_name("wlan0") {
        addres.push_str("-");
        addres.push_str(addr.to_string().replace(":", "").as_str());
    } else if is_none {
        return fail_from_str("无法获取（windows有线无线）mac地址！");
    }
    Ok(addres)
}

pub fn get_index_group_str(cap: &Captures, index: usize) -> Result<String> {
    if let Some(abc) = cap.get(index) {
        return Ok(abc.as_str().to_string());
    }
    fail(format!("无法获取匹配group（{}）的字符串！", index))
}

pub fn get_ip() -> Result<String> {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if let Ok(()) = socket.connect("8.8.8.8:80") {
            if let IpAddr::V4(ip) = socket.local_addr()?.ip() {
                return Ok(ip.to_string());
            }
        }
    };
    fail_from_str("无法获取本地ip4")
}

// impl<T> From<FailType> for Result<T> {
//     fn from(a: FailType) -> Self {
//         return Err(a);
//     }
// }
//
// impl<Ipv4Addr> From<FailType> for Result<Ipv4Addr> {
//     fn from(a: FailType) -> Self {
//         return Err(a);
//     }
// }

/// 用于处理下载的文件（tmp_file），替代原文件（target_file），更新systemd，重启服务（service_name）
///
/// # Examples
/// deal_file("/opt/iot/iot_gateway_tmp", "/opt/iot/iot_gateway", "watch")
///
pub fn deal_file(tmp_file: &str, target_file: &str) -> Result<()> {
    if let Err(com) = Command::new("mv").arg(tmp_file).arg(target_file).status() {
        error!("{:?}", com);
        return fail(com.to_string());
    };

    if let Err(com) = Command::new("chmod").arg("777").arg(target_file).status() {
        error!("{:?}", com);
        return fail(com.to_string());
    }

    if let Err(com) = Command::new("sudo")
        .arg("systemctl")
        .arg("daemon-reload")
        .status()
    {
        error!("{:?}", com);
        return fail(com.to_string());
    }
    return Ok(());
}

pub fn get_today_string() -> String {
    let now: NaiveDateTime = Local::now().naive_local();
    now.format("%Y-%m-%d").to_string()
}

pub fn get_hour_minuter_string() -> String {
    let now: NaiveDateTime = Local::now().naive_local();
    now.format("%H:%M").to_string()
}

// pub struct CommonError {
//     bt: Backtrace,
//     ftn: FailTypeEnum,
// }

impl FailTypeEnum {
    fn new_and_err_chain(msg: String) -> Self {
        CommonError::new(msg.as_str());
        FailTypeEnum::Fail(msg)
    }
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
struct CommonError {
    bt: Backtrace,
}
impl CommonError {
    fn new(msg: &str) {
        error!(
            "error: {}——{:?}",
            msg,
            CommonError {
                bt: Backtrace::new()
            }
        );
    }
}
impl fmt::Debug for CommonError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full = fmt.alternate();
        let frames_tmp = self.bt.frames().len();
        let mut end = 12usize;
        let mut start = 7usize;
        if frames_tmp < 12 {
            end = frames_tmp;
            start = 0;
        }
        println!("start={}, end={}, len={}", start, end, frames_tmp);
        let (frames, style) = (&self.bt.frames()[start..end], PrintFmt::Full);

        // When printing paths we try to strip the cwd if it exists, otherwise
        // we just print the path as-is. Note that we also only do this for the
        // short format, because if it's full we presumably want to print
        // everything.
        let cwd = std::env::current_dir();
        let mut print_path = move |fmt: &mut fmt::Formatter<'_>, path: BytesOrWideString<'_>| {
            let path = path.into_path_buf();
            if !full {
                if let Ok(cwd) = &cwd {
                    if let Ok(suffix) = path.strip_prefix(cwd) {
                        return fmt::Display::fmt(&suffix.display(), fmt);
                    }
                }
            }
            fmt::Display::fmt(&path.display(), fmt)
        };

        let mut f = BacktraceFmt::new(fmt, style, &mut print_path);
        f.add_context()?;
        for frame in frames {
            f.frame().backtrace_frame(frame)?;
        }
        f.finish()?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum FailTypeEnum {
    SerialInterInactive(String),
    Fail(String),
}

impl From<AddrParseError> for FailTypeEnum {
    fn from(a: AddrParseError) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}

impl From<std::io::Error> for FailTypeEnum {
    fn from(a: std::io::Error) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}
impl From<FromUtf8Error> for FailTypeEnum {
    fn from(a: FromUtf8Error) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}

impl From<mio_httpc::Error> for FailTypeEnum {
    fn from(a: mio_httpc::Error) -> Self {
        warn!("mio_httpc::Error={:?}", a);
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}
impl From<ParseIntError> for FailTypeEnum {
    fn from(a: ParseIntError) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}
impl From<TomlError> for FailTypeEnum {
    fn from(a: TomlError) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}
impl From<roxmltree::Error> for FailTypeEnum {
    fn from(a: roxmltree::Error) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}
impl<T> From<rumqttc::SendError<T>> for FailTypeEnum {
    fn from(a: rumqttc::SendError<T>) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}
impl From<mio_serial::Error> for FailTypeEnum {
    fn from(a: mio_serial::Error) -> Self {
        warn!("mio_httpc::Error={:?}", a);
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}

impl From<ClientError> for FailTypeEnum {
    fn from(a: ClientError) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}

impl From<(usize, &str)> for FailTypeEnum {
    fn from(a: (usize, &str)) -> Self {
        let err = format!("position:{}, err {}", a.0, a.1);
        FailTypeEnum::new_and_err_chain(err)
    }
}

impl From<heim::Error> for FailTypeEnum {
    fn from(a: heim::Error) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}

impl From<MacAddressError> for FailTypeEnum {
    fn from(a: MacAddressError) -> Self {
        return FailTypeEnum::new_and_err_chain(a.to_string());
    }
}

#[test]
fn test_get_today_string() {
    println!("{}", get_today_string());
}

pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn fail_from_str<T>(msg: &str) -> Result<T> {
    return Err(FailTypeEnum::Fail(msg.to_string()));
}
pub fn fail<T>(msg: String) -> Result<T> {
    Err(FailTypeEnum::Fail(msg))
}

pub fn fail_from_option<T>(msg: Option<T>, err: &str) -> Result<T> {
    match msg {
        Some(val) => return Ok(val),
        None => return Err(FailTypeEnum::new_and_err_chain(err.to_string())),
    }
}

pub fn fail_from_result<T, W: Debug>(msg: std::result::Result<T, W>) -> Result<T> {
    match msg {
        Ok(t) => {
            return Ok(t);
        }
        Err(w) => {
            return Err(FailTypeEnum::new_and_err_chain(format!("{:?}", w)));
        }
    }
}

#[cfg(target_family = "unix")]
pub fn init_toml_config() -> Result<Toml> {
    // Ok(Toml::init("/home/pi/iot/config/config.toml")?)
    Ok(Toml::init("./config/config.toml")?)
}

#[cfg(target_family = "windows")]
pub fn init_toml_config() -> Result<Toml> {
    Ok(Toml::init("./config/config.toml")?)
}

#[cfg(target_family = "unix")]
pub fn init_log() {
    log4rs::init_file("./config/log4rs.yaml", Default::default()).unwrap();
    // log4rs::init_file("/home/pi/iot/config/log4rs‘.yaml", Default::default()).unwrap();
}
#[cfg(target_family = "windows")]
pub fn init_log() {
    log4rs::init_file("./config/log4rs.yaml", Default::default()).unwrap();
}

#[test]
fn get_hour_minuter_string_test() {
    println!("{}", get_hour_minuter_string());
}

///
/// 关掉休眠
///
// #[cfg(target_family = "unix")]
// pub fn turn_off_screen_sleep() -> Result<ExitStatus> {
//     //xset -display :0.0 dpms force on/off
//     fail_from_result(
//         Command::new("xset")
//             .arg("-display")
//             .arg(":0.0")
//             .arg("dpms")
//             .arg("force")
//             .arg("on")
//             .spawn()?
//             .wait(),
//     )
// }
// #[cfg(target_family = "windows")]
// pub fn turn_off_screen_sleep() -> Result<ExitStatus> {
//     fail_from_str("该功能不支持windows")
// }
#[cfg(target_family = "windows")]
pub fn turn_on_screen() -> Result<ExitStatus> {
    fail_from_str("该功能不支持windows")
}
#[cfg(target_family = "windows")]
pub fn turn_off_screen() -> Result<ExitStatus> {
    fail_from_str("该功能不支持windows")
}
#[cfg(target_family = "unix")]
pub fn turn_on_screen() -> Result<ExitStatus> {
    // /opt/vc/bin/tvservice -p && sudo systemctl restart display-manager
    // let res = Command::new("tvservice").arg("-p").spawn()?.wait()?;
    //     // if res.success() {
    //     //     if turn_off_screen_sleep()?.success() {
    //     //         return fail_from_result(
    //     //             Command::new("sudo")
    //     //                 .arg("systemctl")
    //     //                 .arg("restart")
    //     //                 .arg("display-manager")
    //     //                 .spawn()?
    //     //                 .wait(),
    //     //         );
    //     //     } else {
    //     //         return fail_from_str("xset -display :0.0 dpms force on 执行异常！");
    //     //     }
    //     // } else {
    //     //     return fail_from_str("tvservice -p 执行异常！");
    //     // }
    fail_from_result(
        Command::new("xset")
            .arg("-display")
            .arg(":0.0")
            .arg("dpms")
            .arg("force")
            .arg("on")
            .spawn()?
            .wait(),
    )
}
#[cfg(target_family = "unix")]
pub fn turn_off_screen() -> Result<ExitStatus> {
    // fail_from_result(Command::new("tvservice").arg("-o").spawn()?.wait())
    fail_from_result(
        Command::new("xset")
            .arg("-display")
            .arg(":0.0")
            .arg("dpms")
            .arg("force")
            .arg("off")
            .spawn()?
            .wait(),
    )
}

///
/// 获取2位小数位的f64，超过部分直接截断（不进行四舍五入）
pub fn get_format_f64(before: f64) -> f64 {
    return f64::trunc(before * 100.0) / 100.0;
}

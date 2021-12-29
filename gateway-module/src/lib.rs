use async_std::channel::Sender;
use serde::Deserialize;
mod mconfig;
mod mdaemon;
mod mmqtt;
mod mserial;
mod mwasmer;
mod pub_use;

use crate::pub_use::*;
use config::Value;
pub use mconfig::MConfig;
pub use mdaemon::MDaemon;

pub trait Module {
    // 配置初始化
    // 模块初始化
    // fn init(core: Sender<ModuleCommand>) -> (Self, Sender<CoreCommand>);
    // 模块启动
    fn start(self);
}
#[derive(Debug)]
pub enum ModuleCommand {}

#[derive(Debug)]
pub enum CoreCommand {}

#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    name: Option<String>,
    ty: ModuleKind,
    config: Value,
}

#[derive(Debug, Deserialize)]
#[serde(from = "String")]
enum ModuleKind {
    Mqtt,
    Daemon,
    NonSupport,
}
impl From<String> for ModuleKind {
    fn from(val: String) -> Self {
        let lowercase = val.to_lowercase();
        let name = lowercase.as_str();
        match name {
            "mqtt" => ModuleKind::Mqtt,
            "daemon" => ModuleKind::Daemon,
            _ => ModuleKind::NonSupport,
        }
    }
}

#[test]
fn test_module_config() {
    let c = MConfig::init();
    let modules = c.get_array("module").unwrap();
    for val in modules {
        let mc: ModuleConfig = val.try_into().unwrap();
        // let val = val.into_table().unwrap();
        println!("module: {:?}", mc);
    }
}

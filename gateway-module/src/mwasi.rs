use crate::pub_use::*;
use crate::{CoreCommand, Module, ModuleCommand};
use config::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};

pub struct Wasis {
    name: String,
    config: Vec<WasiConfig>,
    /// 发送至core的指令
    core: Sender<ModuleCommand>,
    /// 接收核心的指令
    recv: Receiver<CoreCommand>,
}

impl Wasis {
    // pub fn new(config: HashMap<String, Value>, core: Sender<ModuleCommand>, recv: Receiver<CoreCommand>) -> Result<Self> {
    //     // let name = config.get("").
    // }
}

impl Module for Wasis {
    fn start(self) {
        // let Self { config, core, recv } = self;
        // task::spawn(async move {
        //     start(config, core, recv).await;
        // });
    }
}

struct WasiConfig {
    name: String,
    // path: Path,
}

async fn start(config: Vec<WasiConfig>, core: Sender<ModuleCommand>, recv: Receiver<CoreCommand>) {}

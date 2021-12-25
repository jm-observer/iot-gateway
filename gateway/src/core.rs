use crate::pub_use::*;

use std::collections::HashMap;

pub struct Core {
    recv: Receiver<ModuleCommand>,
    sender: Sender<ModuleCommand>,
    senders: HashMap<String, Sender<CoreCommand>>,
}

impl Core {
    pub fn init() -> Self {
        let (sender, recv) = async_std::channel::unbounded();
        Self {
            recv,
            sender,
            senders: HashMap::with_capacity(20),
        }
    }
    pub fn start(&mut self) -> Result<()> {
        // 读取配置
        let config = MConfig::init();
        // 依配置初始化模块
        // 守护进程
        let (daemon, sender) = MDaemon::init(self.sender.clone());
        self.senders.insert("daemon".to_string(), sender);
        daemon.start();

        Ok(())
    }
}

use crate::pub_use::*;
use crate::*;

pub struct MDaemon {
    core: Sender<ModuleCommand>,
    recv: Receiver<CoreCommand>,
}

impl MDaemon {
    pub fn init(core: Sender<ModuleCommand>) -> (Self, Sender<CoreCommand>) {
        let (sender, recv) = async_std::channel::unbounded();
        (Self { core, recv }, sender)
    }
}

impl Module for MDaemon {
    fn start(self) {
        let MDaemon { core, recv } = self;
        let handler = task::spawn(async move {
            if let Err(e) = daemon(core, recv).await {
                error!("{:?}", e);
            }
        });
    }
}

/// 守护进程
#[cfg(target_family = "windows")]
pub async fn daemon(_core: Sender<ModuleCommand>, _recv: Receiver<CoreCommand>) -> Result<()> {
    // loop {
    //     if let Ok(res) = recv.recv().await {
    //         debug!("{:?}", res);
    //     }
    // }
    Ok(())
}
/// 守护进程
// #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
#[cfg(target_family = "unix")]
pub async fn daemon(_core: Sender<ModuleCommand>, _recv: Receiver<CoreCommand>) -> Result<()> {
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

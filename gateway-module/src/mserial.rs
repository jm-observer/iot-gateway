use crate::pub_use::*;
use crate::*;
use mio::{Poll, PollOpt, Ready, Token};
use mio_serial::Serial;

pub struct MSerial {
    interface: String,
    /// 发送至core的指令
    core: Sender<ModuleCommand>,
    /// 接收核心的指令
    recv: Receiver<CoreCommand>,
}

impl Module for MSerial {
    fn start(self) {
        let Self {
            interface,
            core,
            recv,
        } = self;
        task::spawn(async move { start(interface, core, recv).await });
    }
}

async fn start(mut interface: String, core: Sender<ModuleCommand>, recv: Receiver<CoreCommand>) {
    let (mut serial, mut poll) = init(interface.as_str(), core.clone());
    loop {
        match recv.recv().await {
            Ok(data) => {}
            Err(e) => {}
        }
    }
}
fn init(interface: &str, core: Sender<ModuleCommand>) -> (Option<Serial>, Option<Poll>) {
    match init_detail(interface) {
        Ok((s, p)) => return (Some(s), Some(p)),
        Err(e) => {
            // todo
            todo!();
            return (None, None);
        }
    }
}

fn init_detail(interface: &str) -> Result<(Serial, Poll)> {
    let poll = Poll::new()?;
    let settings = mio_serial::SerialPortSettings::default();
    let rx = mio_serial::Serial::from_path(interface, &settings)?;
    poll.register(&rx, SERIAL_TOKEN, ready_of_interest(), PollOpt::edge())?;
    Ok((rx, poll))
}

const SERIAL_TOKEN: Token = Token(0);

#[cfg(unix)]
fn ready_of_interest() -> Ready {
    Ready::readable() | UnixReady::hup() | UnixReady::error()
}

#[cfg(windows)]
fn ready_of_interest() -> Ready {
    Ready::readable()
}

#[cfg(unix)]
fn is_closed(state: Ready) -> bool {
    state.contains(UnixReady::hup() | UnixReady::error())
}

#[cfg(windows)]
fn is_closed(_: Ready) -> bool {
    false
}

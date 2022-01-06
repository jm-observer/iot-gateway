use std::io;
use std::io::{Read, Write};
use std::time::Duration;
use crate::pub_use::*;
use crate::*;
use mio::{event::Source, Events, Interest, Poll, Token};
use mio_serial::{SerialPortBuilderExt, SerialStream};

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

async fn start(interface: String, core: Sender<ModuleCommand>, recv: Receiver<CoreCommand>) {
    let (serial, poll) = init(interface.as_str(), core.clone());
    loop {
        match recv.recv().await {
            Ok(data) => {}
            Err(e) => {}
        }
    }
}
fn init(interface: &str, core: Sender<ModuleCommand>) -> (Option<SerialStream>, Option<Poll>) {
    match init_detail(interface) {
        Ok((s, p)) => return (Some(s), Some(p)),
        Err(e) => {
            // todo
            todo!();
            return (None, None);
        }
    }
}
const DEFAULT_BAUD: u32 = 9600;
fn init_detail(interface: &str) -> Result<(SerialStream, Poll)> {
    // let mut poll = Poll::new()?;
    // let mut rx: mio_serial::SerialStream =
    //     mio_serial::new(interface, DEFAULT_BAUD).open_native_async()?;
    // // let mut rx = mio_serial::COMPort::open(&builder)?;
    // poll.registry()
    //     .register(&mut rx, SERIAL_TOKEN, Interest::WRITABLE)
    //     .unwrap();
    let mut poll = Poll::new()?;
    let mut rx = mio_serial::new(interface, DEFAULT_BAUD).open_native_async()?;
    poll.registry()
        .register(&mut rx, SERIAL_TOKEN, Interest::WRITABLE)
        .unwrap();
    Ok((rx, poll))
}

const SERIAL_TOKEN: Token = Token(0);


const ELE_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x04, 0xC5, 0xCB];
// const ELE_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x31, 0x84, 0x1E];
// 报错 range end index 7 out of range for slice of length 5
// thread 'mserial::test' panicked at 'range end index 7 out of range for slice of length 5', gateway-module\src\mserial.rs:81:26
// const ELE_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x32, 0xC4, 0x1F];


pub fn ele_info(interface: &str) -> crate::Result<(f64, f64, f64, f64)> {
    let (mut rx, mut poll) = init_detail(interface).unwrap();
    let res = get_serial_val(&mut rx, &mut poll, &ELE_INPUT)?;
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
        bail!("接口读错：疑似读取温湿气压传感器");
    }
    tmp.copy_from_slice(&res[9..11]);
    let d = u16::from_be_bytes(tmp) as f64 * 0.01;
    Ok((a, b, c, d))
}

#[test]
fn test() {
    let (a1, a2, a3, a4) = ele_info("COM4").unwrap();
    println!("{}", a1);
    println!("{}", a2);
}


pub fn get_serial_val(rx: &mut SerialStream, poll: &mut Poll, input: &[u8]) -> crate::Result<Vec<u8>> {
    let mut rx_buf = [0u8; 1024];
    let mut events = Events::with_capacity(1);
    rx.write_all(input)?;
    loop {
        poll.poll(&mut events, None)?;
        for event in events.iter() {
            match event.token() {
                SERIAL_TOKEN => loop {
                    match rx.read(&mut rx_buf) {
                        Ok(count) => {
                            return Ok(rx_buf[..count].to_vec());
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            bail!("Quitting due to read error: {}", e);
                        }
                    }
                },
                t => bail!("Unexpected token: {:?}", t),
            }
        }
    }
}

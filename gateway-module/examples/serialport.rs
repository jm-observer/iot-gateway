//! Simple example that echoes received serial traffic to stdout
extern crate mio;
extern crate mio_serial;

use mio::{Events, Interest, Poll, Token};

use std::env;
use std::io;
use std::io::{Read, Write};
use std::str;

use mio_serial::SerialPortBuilderExt;
use gateway_module::{get_serial_val, init_detail};

const SERIAL_TOKEN: Token = Token(0);

#[cfg(windows)]
const DEFAULT_TTY: &str = "COM4";

const DEFAULT_BAUD: u32 = 9600;
const ELE_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x04, 0xC5, 0xCB];
pub fn main() -> io::Result<()> {

    let path = DEFAULT_TTY.to_string();
    println!("Opening {} at 9600,8N1", path);
    // let mut poll = Poll::new()?;
    // let mut rx = mio_serial::new(path, DEFAULT_BAUD).open_native_async()?;
    // poll.registry()
    //     .register(&mut rx, SERIAL_TOKEN, Interest::WRITABLE)
    //     .unwrap();

    use crc::{Crc, CRC_16_MODBUS};
    let X25: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
    let mut input = vec![0x01u8, 0x03, 0x00, 0x00, 0x00, 0x40];
    input.extend_from_slice(change_to_u8(X25.checksum(&input.as_slice())).as_slice());
    // [1, 3, 0, 0, 0, 64, 68, 58]
    println!("{:?}", input);
    println!("{:?}", ELE_INPUT);


    let (mut rx, mut poll) = init_detail(path.as_str()).unwrap();

    let res = get_serial_val(&mut  rx, &mut poll, input.as_slice()).unwrap();
    println!("len = {:?} {:?}", res.len(), res[2]);
    Ok(())
    // let mut buf = [0u8; 1024];
    // let mut events = Events::with_capacity(1);
    // rx.write_all(&ELE_INPUT);
    // loop {
    //     poll.poll(&mut events, None)?;
    //     for event in events.iter() {
    //         match event.token() {
    //             SERIAL_TOKEN => loop {
    //                 match rx.read(&mut buf) {
    //                     Ok(count) => {
    //                         println!("111{:?}", String::from_utf8_lossy(&buf[..count]))
    //                     }
    //                     Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    //                         break;
    //                     }
    //                     Err(e) => {
    //                         println!("Quitting due to read error: {}", e);
    //                         return Err(e);
    //                     }
    //                 }
    //             },
    //             _ => {
    //                 // This should never happen as we only registered our
    //                 // `UdpSocket` using the `UDP_SOCKET` token, but if it ever
    //                 // does we'll log it.
    //                 // warn!("Got event for unexpected token: {:?}", event);
    //             }
    //         }
    //     }
    // }
}

fn change_to_u8(data: u16) -> [u8; 2] {
    /// as 直接截断
    [data.clone()  as u8, (data >> 8) as u8]
}

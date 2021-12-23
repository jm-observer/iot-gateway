#[cfg(unix)]
use mio::unix::UnixReady;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio_serial::Serial;
use serialport::SerialPortSettings;
use std::io;
use std::io::{Read, Write};

fn main() {
    // ele_info("/dev/ttyS0");
    test_serial();
}

const ELE_INPUTS: [u8; 8] = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x041, 0xC5, 0xCB];

fn test_serial() {
    let port2_name = "/dev/ttyUSB0";
    // Run single-port tests on port1
    let mut port = match serialport::open(port2_name) {
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port2_name, e);
            ::std::process::exit(1);
        }
        Ok(p) => p,
    };
    let port_settings: SerialPortSettings = Default::default();
    port.set_all(&port_settings)
        .expect("Resetting port to sane defaults failed");
    port.write_all(&ELE_INPUTS).expect("Unable to write bytes.");
    println!("success");

    let mut buf = [0u8; 30];
    match port.read(&mut buf) {
        Ok(size) => println!("success={}, {:?}", size, buf),
        Err(e) => println!("{:?}", e),
    }
}

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

const ELE_INPUT: [u8; 8] = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x041, 0xC5, 0xCB];

#[allow(dead_code)]
fn ele_info(inter: &str) -> Result<(f64, f64, f64, f64), String> {
    let (mut rx, poll) = _get_serial(inter).unwrap();
    let res = get_serial_val(&mut rx, &poll, &ELE_INPUT).unwrap();

    drop(rx);
    drop(poll);
    if res.len() < 13 {
        println!("{:?}", res.len());
        return Err("读取结果长度不对！".to_string());
    }
    let mut tmp = [0u8; 2];
    tmp.copy_from_slice(&res[3..5]);
    let a = u16::from_be_bytes(tmp) as f64 * 0.01;
    tmp.copy_from_slice(&res[5..7]);
    let b = u16::from_be_bytes(tmp) as f64 * 0.01;
    tmp.copy_from_slice(&res[7..9]);
    let c = u16::from_be_bytes(tmp) as f64 * 0.01;
    tmp.copy_from_slice(&res[9..11]);
    let d = u16::from_be_bytes(tmp) as f64 * 0.01;
    Ok((a, b, c, d))
}
const SERIAL_TOKEN: Token = Token(0);
fn _get_serial(inter: &str) -> Result<(Serial, Poll), ()> {
    let poll = Poll::new().unwrap();
    let settings = mio_serial::SerialPortSettings::default();
    let rx = mio_serial::Serial::from_path(inter, &settings).unwrap();
    poll.register(&rx, SERIAL_TOKEN, ready_of_interest(), PollOpt::edge())
        .unwrap();
    Ok((rx, poll))
}

fn get_serial_val(rx: &mut Serial, poll: &Poll, input: &[u8]) -> Result<Vec<u8>, String> {
    let mut rx_buf = [0u8; 1024];
    let mut events = Events::with_capacity(1024);
    rx.write_all(input).unwrap();
    // loop {
    // poll.poll(&mut events, None).unwrap();
    // if events.is_empty() {
    //     continue;
    // }
    'outer: loop {
        if let Err(ref e) = poll.poll(&mut events, None) {
            println!("poll failed: {}", e);
            break;
        }
        if events.is_empty() {
            println!("Read timed out!");
            continue;
        }

        for event in events.iter() {
            match event.token() {
                SERIAL_TOKEN => {
                    let ready = event.readiness();
                    // if is_closed(ready) {
                    //     return Err(format!("Quitting due to event: {:?}", ready));
                    // }
                    if is_closed(ready) {
                        println!("Quitting due to event: {:?}", ready);
                        break 'outer;
                    }

                    // if ready.is_readable() {
                    //     loop {
                    //         match rx.read(&mut rx_buf) {
                    //             Ok(count) => {
                    //                 return Ok(Vec::from(&rx_buf[..count]));
                    //             }
                    //             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    //                 return Err(format!("Quitting due to read error: {}", e));
                    //             }
                    //             Err(ref e) => {
                    //                 return Err(format!("Quitting due to read error: {}", e));
                    //             }
                    //         }
                    //     }
                    // } else {
                    //     continue;
                    // }
                    if ready.is_readable() {
                        // With edge triggered events, we must perform reading until we receive a WouldBlock.
                        // See https://docs.rs/mio/0.6/mio/struct.Poll.html for details.
                        loop {
                            match rx.read(&mut rx_buf) {
                                Ok(count) => {
                                    println!("{:?}", String::from_utf8_lossy(&rx_buf[..count]))
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    break;
                                }
                                Err(ref e) => {
                                    println!("Quitting due to read error: {}", e);
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
                t => return Err(format!("Unexpected token: {:?}", t)),
            }
        }
    }
    Err("循环外异常".to_string())
}

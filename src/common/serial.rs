/// 串口通讯的相关操作
use crate::*;
#[cfg(unix)]
use mio::unix::UnixReady;
use mio::{Events, Poll, PollOpt, Ready, Token};
// use mio_serial::*;
use mio_serial::Serial;
use std::io;
use std::io::{Read, Write};
use std::str;
use std::time::Duration;

pub async fn get_serial(interface: &str, milliseconds: u64) -> crate::Result<(Serial, Poll)> {
    let poll = Poll::new()?;
    let settings = mio_serial::SerialPortSettings::default();
    let rx = match mio_serial::Serial::from_path(interface, &settings) {
        Ok(rx) => rx,
        Err(_) => {
            task::sleep(Duration::from_millis(milliseconds)).await;
            mio_serial::Serial::from_path(interface, &settings)?
        }
    };
    poll.register(&rx, SERIAL_TOKEN, ready_of_interest(), PollOpt::edge())?;
    Ok((rx, poll))
}

pub fn get_serial_val(rx: &mut Serial, poll: &Poll, input: &[u8]) -> crate::Result<Vec<u8>> {
    let mut rx_buf = [0u8; 1024];
    let mut events = Events::with_capacity(1024);
    rx.write_all(input)?;
    let mut time = 0u32;
    let timeout = 3u32;
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;
        if events.is_empty() {
            // return fail_from_str("Serial Read timed out!");
            time = time + 1u32;
            if time < timeout {
                continue;
            } else {
                return fail_from_str("Serial Read timed out!");
            }
        }
        for event in events.iter() {
            match event.token() {
                SERIAL_TOKEN => {
                    let ready = event.readiness();
                    if is_closed(ready) {
                        return fail(format!("Quitting due to event: {:?}", ready));
                    }
                    if ready.is_readable() {
                        loop {
                            match rx.read(&mut rx_buf) {
                                Ok(count) => {
                                    return Ok(Vec::from(&rx_buf[..count]));
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    return fail(format!("Quitting due to read error: {}", e));
                                }
                                Err(ref e) => {
                                    return fail(format!("Quitting due to read error: {}", e));
                                }
                            }
                        }
                    } else {
                        continue;
                    }
                }
                t => return fail(format!("Unexpected token: {:?}", t)),
            }
        }
    }
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

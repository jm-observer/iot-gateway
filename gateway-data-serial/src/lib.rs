use anyhow::{bail, Result};
use crc::{Crc, CRC_16_MODBUS};
use log::{debug, warn};
use mio::{Events, Interest, Poll, Token};
use mio_serial::SerialPortBuilderExt;
use std::io;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::time::Duration;

const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
const DEFAULT_BAUD: u32 = 9600;
const SERIAL_TOKEN: Token = Token(0);

pub trait BuildSerialQuery<'a>: From<Vec<u8>> {
    fn build(interface: &'a str) -> SerialQuery<'a, Self>;
}

/// 采集参数
pub struct SerialQuery<'a, T>
where
    T: BuildSerialQuery<'a>,
{
    data: [u8; 8],
    interface: &'a str,
    marker: PhantomData<T>,
}

/// 采集结果
struct SerialResult {
    data: Vec<u8>,
}

impl<'a, T> SerialQuery<'a, T>
where
    T: BuildSerialQuery<'a>,
{
    /// 会重置校验码
    pub fn build(mut data: [u8; 8], interface: &'a str) -> Self {
        let check_num = change_to_u8(X25.checksum(&data[0..6]));
        data[6] = check_num[0];
        data[7] = check_num[1];
        Self {
            data,
            interface,
            marker: PhantomData,
        }
    }
    /// 不再生成校验码
    pub fn build_uncheck(data: [u8; 8], interface: &'a str) -> Self {
        debug!("data: {:?}, interface: {:?}", data, interface);
        Self {
            data,
            interface,
            marker: PhantomData,
        }
    }
    pub fn new(
        addr: u8,
        func: u8,
        start_addr: [u8; 2],
        register_length: [u8; 2],
        interface: &'a str,
    ) -> Self {
        let mut data = [
            addr,
            func,
            start_addr[0],
            start_addr[1],
            register_length[0],
            register_length[1],
            0,
            0,
        ];
        let check_num = change_to_u8(X25.checksum(&data[0..6]));
        data[6] = check_num[0];
        data[7] = check_num[1];
        Self {
            data,
            interface,
            marker: PhantomData,
        }
    }
    pub fn from_vec(data: Vec<u8>, interface: &'a str) -> Result<Self> {
        if data.len() < 8 {
            bail!("数据长度不足：{:?}！", data.len());
        } else if data.len() > 8 {
            warn!("数据长达大于8：{:?}", data.len());
        }
        let mut input = [0u8; 8];
        for i in 0..8 {
            input[i] = data[i];
        }
        Ok(Self {
            data: input,
            interface,
            marker: PhantomData,
        })
    }

    pub fn collect(&self) -> Result<T> {
        let res = SerialResult::new(self.collect_data()?, self.register_len())?;
        Ok(T::try_from(res.data)?)
    }

    fn collect_data(&self) -> Result<Vec<u8>> {
        let mut poll = Poll::new()?;
        let mut rx = mio_serial::new(self.interface, DEFAULT_BAUD).open_native_async()?;
        poll.registry()
            .register(&mut rx, SERIAL_TOKEN, Interest::WRITABLE)
            .unwrap();
        // 不能用vec，否则读取的长度为0
        let mut tmp = [0u8; 1024];
        let mut events = Events::with_capacity(1);
        let res_len = self.res_len();
        let mut tmp_len = 0;
        let mut start: &mut [u8] = &mut tmp;
        rx.write_all(&self.data)?;
        loop {
            poll.poll(&mut events, Some(Duration::from_millis(1000)))?;
            for event in events.iter() {
                debug!("event.token={:?}", event.token());
                match event.token() {
                    SERIAL_TOKEN => loop {
                        match rx.read(start) {
                            Ok(count) => {
                                debug!("read.count: {:?}", count);
                                tmp_len += count;
                                if tmp_len >= res_len {
                                    return Ok(tmp[..res_len].to_vec());
                                } else {
                                    start = &mut tmp[tmp_len..];
                                    continue;
                                }
                                // return Ok(tmp[..count].to_vec());
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

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
    pub fn register_len(&self) -> usize {
        change_to_usize([self.data[4], self.data[5]])
    }
    pub fn res_len(&self) -> usize {
        change_to_usize([self.data[4], self.data[5]]) * 2 + 5
    }
}

impl SerialResult {
    pub fn new(data: Vec<u8>, query_len: usize) -> Result<Self> {
        // 地址1、功能1、数据长度1、数据>= 1、校验码2、
        if data.len() <= 6 {
            bail!("数据长度不足：{:?}", data.len());
        }
        if data[2] as usize != query_len * 2 {
            bail!(
                "寄存器数据长度不符合：{:?}, expected: {:?}",
                data[2] as usize,
                query_len
            );
        }
        let len = query_len * 2 + 5;
        if data.len() != len {
            bail!("总数据长度不符合：{:?}, expected: {:?}", data.len(), len);
        }
        let check_num = change_to_u8(X25.checksum(&data[0..(len - 2)]));
        if check_num[0] != data[len - 2] || check_num[1] != data[len - 1] {
            bail!(
                "校验码不符合：{:?}, expected: {:?}",
                check_num,
                [data[len - 2], data[len - 1]]
            );
        }
        Ok(Self { data })
    }
}

// impl Default
fn change_to_u8(data: u16) -> [u8; 2] {
    // as 直接截断
    // 低位在前，高位在后
    [data as u8, (data >> 8) as u8]
}

fn change_to_usize(data: [u8; 2]) -> usize {
    // println!("{:?}, {:?}", (data[0] as u16) << 8, )
    ((data[0] as u16) << 8 | (data[1] as u16)) as usize
}

/// 根据从设备号，读取长度生成输入参数（起始地址为0x0000， 功能码为0x03）
pub fn general_read_input(dev_no: u8, register_len: [u8; 2]) -> [u8; 8] {
    let mut data = [dev_no, 3, 0, 0, register_len[0], register_len[1], 0, 0];
    let check_num = change_to_u8(X25.checksum(&data[0..6]));
    data[6] = check_num[0];
    data[7] = check_num[1];
    data
}

#[test]
fn test_change_to_usize() {
    assert_eq!(change_to_usize([0x0, 0x40]), 64);
}

#[test]
fn test_serial_query() {
    let vec_data = vec![1u8, 3, 0, 0, 0, 64, 68, 58];
    let query0 = SerialQuery::from_vec(vec_data).unwrap();

    let data = [1, 3, 0, 0, 0, 64];
    let query1 = SerialQuery::new(data[0], data[1], [data[2], data[3]], [data[4], data[5]]);
    assert_eq!(query0.data(), query1.data());
    assert_eq!(query0.register_len(), 64usize);
}

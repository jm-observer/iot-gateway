use anyhow::{bail, Result};
use crc::{Crc, CRC_16_MODBUS};
use log::{debug, warn};

const  X25: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
pub struct SerialQuery {
    data: [u8; 8],
}

pub struct SerialResult {
    data: Vec<u8>
}

impl SerialResult {
    pub fn new(data: Vec<u8>, query_len: usize, ) -> Result<Self> {
        // 地址1、功能1、数据长度1、数据>= 1、校验码2、
        if data.len() <= 6 {
            bail!("数据长度不足：{:?}", data.len());
        }
        if data[2] as usize != query_len {
            bail!("寄存器数据长度不符合：{:?}, expected: {:?}", data[2] as usize, query_len);
        }
        let len = query_len + 5;
        if data.len() != len {
            bail!("总数据长度不符合：{:?}, expected: {:?}", data.len(), len);
        }
        let check_num = change_to_u8(X25.checksum(&data[0..(len-2)]));
        if check_num[0] != data[len - 2] || check_num[1] != data[len - 1] {
            bail!("校验码不符合：{:?}, expected: {:?}", check_num, [data[len - 2], data[len - 1]]);
        }
        Ok(Self {
            data
        })
    }
}

impl SerialQuery {
    pub fn new(addr: u8, func: u8, start_addr: [u8; 2], register_length: [u8; 2]) -> Self {
        let mut data = [addr, func, start_addr[0], start_addr[1], register_length[0], register_length[1], 0, 0];
        let check_num = change_to_u8(X25.checksum(&data[0..6]));
        data[6] = check_num[0];
        data[7] = check_num[1];
        Self {
            data
        }
    }
    pub fn from_vec(data: Vec<u8>) -> Result<Self> {
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
            data: input
        })
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
    pub fn register_length(&self) -> usize {
        change_to_usize([self.data[4], self.data[5]])
    }
}
impl Default for SerialQuery {
    fn default() -> Self {
        Self {
            data: [0u8; 8]
        }
    }
}
fn change_to_u8(data: u16) -> [u8; 2] {
    /// as 直接截断
    /// 低位在前，高位在后
    [data as u8, (data >> 8) as u8]
}


fn change_to_usize(data: [u8; 2]) -> usize {
    // println!("{:?}, {:?}", (data[0] as u16) << 8, )
    ((data[0] as u16) << 8 | (data[1] as u16) ) as usize
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
    assert_eq!(query0.register_length(), 64usize);
}
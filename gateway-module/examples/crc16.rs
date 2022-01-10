use std::sync::atomic::AtomicI32;
use crc::{Crc, Algorithm, CRC_16_MODBUS, CRC_32_ISCSI};
use serde::__private::de::Content::I32;

pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
pub fn main() {
    // 电流检测
    let data = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x04];
    assert_eq!(change_to_u8(X25.checksum(&data)), [0xcb, 0xc5]);
    // 大气压
    let data = [0x01u8, 0x03, 0x00, 0x0B, 0x00, 0x01];
    assert_eq!(change_to_u8(X25.checksum(&data)), [0xc8, 0xf5]); //0xc8f5
}

fn change_to_u8(data: u16) -> [u8; 2] {
    /// as 直接截断
    [data.clone()  as u8, (data >> 8) as u8]
}
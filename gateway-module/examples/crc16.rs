use crc::{Crc, Algorithm, CRC_16_MODBUS, CRC_32_ISCSI};
pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
pub fn main() {
    // 电流检测
    let data = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x04];
    assert_eq!(X25.checksum(&data), 0xcbc5);
    // 大气压
    let data = [0x01u8, 0x03, 0x00, 0x0B, 0x00, 0x01];
    assert_eq!(X25.checksum(&data), 0xc8f5);
    //
    // let data = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x0B];
    // assert_eq!(X25.checksum(&data), 0xc8f5);
}
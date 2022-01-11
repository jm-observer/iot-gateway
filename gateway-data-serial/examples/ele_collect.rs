use anyhow::Result;
use gateway_data_serial::{general_read_input, BuildSerialQuery, SerialQuery};
use log::debug;

#[async_std::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let query = EleInfo::build(INTERFACE_COM4);
    let env = query.collect()?;
    debug!("{:?}", env);
    Ok(())
}

#[derive(Debug)]
pub struct EleInfo {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

static INTERFACE_COM4: &str = "COM4";

impl From<Vec<u8>> for EleInfo {
    fn from(value: Vec<u8>) -> Self {
        // 调用前已判断过长度了
        let a = u16::from_be_bytes([value[19], value[20]]) as f64 * 0.1;
        let b = u16::from_be_bytes([value[21], value[22]]) as f64 * 0.1;
        let c = u16::from_be_bytes([value[23], value[24]]) as f64 * 0.1;
        let d = u16::from_be_bytes([value[25], value[26]]) as f64 * 0.1;

        EleInfo { a, b, c, d }
    }
}

impl<'a> BuildSerialQuery<'a> for EleInfo {
    fn build(interface: &'a str) -> SerialQuery<'a, Self> {
        // SerialQuery::build_uncheck(general_read_input(0x01, [0x00, 0x40]), interface)
        SerialQuery::build_uncheck(general_read_input(0x01, [0x00, 0x30]), interface)
    }
}

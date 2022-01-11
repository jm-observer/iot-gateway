use anyhow::Result;
use gateway_data_serial::{general_read_input, BuildSerialQuery, SerialQuery};
use log::debug;

#[async_std::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let query = EnvInfo::build(INTERFACE_COM4);
    let env = query.collect()?;
    debug!("{:?}", env);
    Ok(())
}

#[derive(Debug)]
pub struct EnvInfo {
    temperature: f64,
    humidity: f64,
    air_pressure: f64,
}

static INTERFACE_COM4: &str = "COM4";

impl From<Vec<u8>> for EnvInfo {
    fn from(value: Vec<u8>) -> Self {
        // 调用前已判断过长度了
        let temperature = u16::from_be_bytes([value[5], value[6]]) as f64 * 0.1;
        let humidity = u16::from_be_bytes([value[3], value[4]]) as f64 * 0.1;
        let air_pressure = u16::from_be_bytes([value[25], value[26]]) as f64 * 0.1;
        EnvInfo {
            temperature,
            humidity,
            air_pressure,
        }
    }
}

impl<'a> BuildSerialQuery<'a> for EnvInfo {
    fn build(interface: &'a str) -> SerialQuery<'a, Self> {
        // SerialQuery::build_uncheck(general_read_input(0x01, [0x00, 0x40]), interface)
        SerialQuery::build_uncheck(general_read_input(0x01, [0x00, 0x40]), interface)
    }
}

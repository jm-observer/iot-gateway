fn main() {}

async fn env_info(res: Vec<u8>) -> crate::Result<(f64, f64, f64)> {
    todo!();
    let mut tmp = [0u8; 2];
    tmp.copy_from_slice(&res[3..5]);
    let air = u16::from_be_bytes(tmp) as f64 * 0.1;
    //读取电流传感器的数值也许是：{"airp":0,"humidity":0.65,"temp":4,"u":"1f10a416-8679-48ea-b245-611196b5cf87"}
    if air < 300f64 {
        bail!("接口读错：疑似读取电流传感器");
    }
    tmp.copy_from_slice(&res[3..5]);
    let humidity = u16::from_be_bytes(tmp) as f64 * 0.001;
    tmp.copy_from_slice(&res[5..7]);
    let temp = u16::from_be_bytes(tmp) as f64 * 0.1;
    Ok((air, humidity, temp))
}

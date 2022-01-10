wit_bindgen_rust::export!("./config/wit/serial.wit");

pub struct Serial;

//cargo build --package wasi-env --target wasm32-wasi --release
/// 温湿压传感器的输入、输出
impl serial::Serial for Serial {
    fn res(_param: Vec<u8>) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
    fn input() -> Vec<u8> {
        vec![1u8, 3, 0, 0, 0, 64, 68, 58]
    }
}


// get_input: function() -> list<u8>
// get_res: function(param: list<u8>) -> expected<list<u8>, string>
wit_bindgen_rust::export!("./config/wit/wasi.wit");
struct Wasi;

impl wasi::Wasi for Wasi {
    fn wasi(_param: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        Ok(Vec::new())
    }
}
/// cargo expand --package gateway-wasi-data --bin watch-source
fn main() {}

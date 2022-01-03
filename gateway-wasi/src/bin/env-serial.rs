use gateway_wasi::setup;

fn main() -> anyhow::Result<()> {
    log::trace!("Setting up everything.");
    let (mut store, plugin) = setup("config/wasi/gateway_wasi_data.wasm")?;
    let res = plugin
        .wasi(&mut store, vec![0, 159, 146, 150].as_slice())
        .unwrap();
    // println!(
    //     "Welcome to the {:?} plugin!",
    //     String::from_utf8(res).unwrap()
    // );
    Ok(())
}

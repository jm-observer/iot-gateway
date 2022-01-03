use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

wit_bindgen_wasmtime::import!("./config/wit/wasi.wit");
use wasi::*;

pub fn setup(
    wasi: &str,
) -> anyhow::Result<(Store<(WasiCtx, WasiData)>, Wasi<(WasiCtx, WasiData)>)> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, wasi)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |(wasi, _plugin_data)| wasi)?;
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, (wasi, WasiData {}));
    let (plugin, _instance) =
        Wasi::instantiate(&mut store, &module, &mut linker, |(_wasi, plugin_data)| {
            plugin_data
        })?;
    Ok((store, plugin))
}

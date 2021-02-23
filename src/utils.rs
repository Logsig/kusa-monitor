use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::{ArrayBuffer, Function, Object, Reflect};
use wasm_mt::prelude::*;

pub fn create_mt(clazz: &Object) -> WasmMt {
    WasmMt::new_with_arraybuffers(
        get_ab(&clazz, "getPkgJs").unwrap(),
        get_ab(&clazz, "getPkgWasm").unwrap())
}

fn get_ab(clazz: &Object, name: &str) -> Result<ArrayBuffer, JsValue> {
    Ok(Reflect::get(clazz, &name.into())?
        .dyn_into::<Function>()?
        .call0(&JsValue::undefined())?
        .into())
}

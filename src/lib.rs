use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let opt = web_sys::window(); // Uncaught (in promise) ReferenceError: Window is not defined

    assert!(opt.is_none());

    Ok(())
}

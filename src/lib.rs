use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
//use js_sys::{ArrayBuffer, Function, Promise, Reflect};
use js_sys::{Reflect};

#[macro_export]
macro_rules! console_ln {
    ( $( $x:expr ),* ) => {
        {
            let ln = format!( $( $x ),* );
            // console::log_1(&ln.into());
            web_sys::console::log_1(&ln.into());
        }
    };
}

fn is_worker() -> bool {
    // let opt = web_sys::window(); // Uncaught (in promise) ReferenceError: Window is not defined

    // TODO in case of Node.js

    let obj = js_sys::global().unchecked_into::<web_sys::Window>();
    // console_ln!("obj: {:?}", obj);

    Reflect::get(&obj, &JsValue::from("window"))
        .unwrap_throw()
        .is_undefined()
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // console_ln!("start(): is_worker: {}", is_worker());

    if is_worker() {
        start_worker()
    } else {
        start_main()
    }
}

fn start_main() -> Result<(), JsValue> {
    console_ln!("start_main(): hi");
    Ok(())
}
fn start_worker() -> Result<(), JsValue> {
    console_ln!("start_worker(): hi");
    Ok(())
}

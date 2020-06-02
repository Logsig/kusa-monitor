#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use js_sys::Reflect;
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, sleep};

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        let _ = entry_main().await;
    });

    spawn_local(async move {
        let mt = WasmMt::new("./pkg/kusa_monitor.js")
            .and_init().await.unwrap();
        let th = mt.thread().and_init().await.unwrap();

        let _ = exec!(th, async move || {
            let _ = entry_worker().await;
            Ok(JsValue::NULL)
        }).await;
    });
}

async fn entry_main() {
    assert!(!is_worker());

    console_ln!("main: hi0");
    sleep(1000).await;
    console_ln!("main: hi1");
}

async fn entry_worker() {
    assert!(is_worker());

    #[allow(warnings)] loop {
        console_ln!("worker: looping");
        sleep(1000).await;
    }
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

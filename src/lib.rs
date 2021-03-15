#![feature(async_closure)]

mod utils;
use crate::utils::{create_mt, is_worker};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use js_sys::Object;
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, Counter, sleep};

mod plotters;
use crate::plotters::DrawResult;

mod plot_thread;

#[wasm_bindgen]
pub fn app(clazz: Object) -> Result<(), JsValue> {
    spawn_local(async move {
        let _ = entry_main().await;
    });

    let mt = create_mt(&clazz);
    spawn_local(async move {
        let th = mt.and_init().await.unwrap()
            .thread().and_init().await.unwrap();

        let _ = exec!(th, async move || {
            let _ = entry_worker().await;
            Ok(JsValue::NULL)
        }).await;
    });

    Ok(())
}

async fn entry_main() {
    assert!(!is_worker());

    console_ln!("main: start");
    sleep(1000).await;
    console_ln!("main: end");
}

async fn entry_worker() {
    assert!(is_worker());

    let counter = Counter::new();
    #[allow(warnings)] loop {
        let ms = 2000;
        console_ln!("worker: looping (every {}ms): {}", ms, counter.inc());
        sleep(ms).await;
        if counter.num() > 4 {
            console_ln!("worker: bye!");
            break;
        }
    }
}

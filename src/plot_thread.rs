use crate::plotters::mandelbrot;
use crate::utils::create_mt;

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use js_sys::Object;
use wasm_mt::prelude::*;
use wasm_mt::utils::console_ln;

#[wasm_bindgen]
pub struct PlotThread {
    th: wasm_mt::Thread,
}

#[wasm_bindgen]
impl PlotThread {
    #[wasm_bindgen(constructor)]
    pub fn new(clazz: &Object) -> Self {
        console_ln!("PlotThread::new(): hi");
        Self { th: create_mt(clazz).thread() }
    }

    pub async fn and_init(self) -> Self {
        self.th.init().await.unwrap();
        self
    }

    // FIXME * -- https://github.com/rustwasm/wasm-bindgen/issues/1858#issuecomment-552108855
    //   pub async fn mandelbrot(&self) { ... }
    pub async fn mandelbrot(self, canvas: HtmlCanvasElement) -> Self {

        // TODO move and call this inside the thread below
        let set = mandelbrot::mandelbrot_set( // dummy params for now
            std::ops::Range { start: 0.0, end: 0.0 },
            std::ops::Range { start: 0.0, end: 0.0 },
            (0, 0), 100);

        // TODO `_set` will be a pixel array
        let _set = exec!(self.th, move || {
            console_ln!("todo: call `mandelbrot_set()` here!!");

            Ok(JsValue::NULL)
        }).await;

        mandelbrot::draw_set(canvas, set).map_err(|err| err.to_string()).unwrap();

        self // FIXME *
    }
}

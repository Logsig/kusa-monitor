use crate::plotters::mandelbrot;
use crate::utils::create_mt;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use js_sys::{ArrayBuffer, Float64Array, Uint32Array, Object};
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
        let root = mandelbrot::create_root(canvas);
        let chart = mandelbrot::create_chart(&root).unwrap();
        let (real, complex, samples) = mandelbrot::get_params(&chart);

        let jsv = exec!(self.th, move || {
            let arr = mandelbrot::mandelbrot_arr_ab(real, complex, samples, 100);

            // TODO transferables !!!!!!!!
            Ok(JsValue::from(arr))
        }).await.unwrap();

        let arr_outer = jsv.dyn_ref::<js_sys::Array>().unwrap();
        let abs = (0..=2)
            .map(|idx| arr_outer.get(idx).dyn_into().unwrap())
            .collect::<Vec<ArrayBuffer>>();
        let arrs = (
            Float64Array::new(&abs[0]),
            Float64Array::new(&abs[1]),
            Uint32Array::new(&abs[2]));

        let len = arrs.0.length();
        let set = (0..len).map(|idx| (
            arrs.0.get_index(idx),
            arrs.1.get_index(idx),
            arrs.2.get_index(idx) as usize,
        ));
        mandelbrot::draw_set(&root, &chart, set, 5).map_err(|err| err.to_string()).unwrap();

        self // FIXME *
    }
}

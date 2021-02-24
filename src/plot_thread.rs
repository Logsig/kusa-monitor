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
        let jsv = exec!(self.th, move || {
            let arr = mandelbrot::mandelbrot_arr_ab(
                // FIXME hardcoded !!!!
                std::ops::Range { start: -2.1, end: 0.6 },
                std::ops::Range { start: -1.2, end: 1.2 },
                (512, 324), 100);

            // TODO transferables !!!!!!!!
            Ok(JsValue::from(arr))
        }).await.unwrap();

        let arr = jsv.dyn_ref::<js_sys::Array>().unwrap();
        let ab_x = arr.get(0).dyn_into::<ArrayBuffer>().unwrap();
        let ab_y = arr.get(1).dyn_into::<ArrayBuffer>().unwrap();
        let ab_c = arr.get(2).dyn_into::<ArrayBuffer>().unwrap();

        //==== ok
        // let vec_x = Float64Array::new(&ab_x).to_vec(); // copied
        // console_ln!("vec_x: {:?}", vec_x);
        //==== ok
        // let arr_x = Float64Array::new(&ab_x);
        // arr_x.for_each(&mut |x, _idx, _arr| console_ln!("x: {}", x));

        let arr_x = Float64Array::new(&ab_x);
        let arr_y = Float64Array::new(&ab_y);
        let arr_c = Uint32Array::new(&ab_c);
        let set = (0..arr_x.length()).map(|idx| (
            arr_x.get_index(idx),
            arr_y.get_index(idx),
            arr_c.get_index(idx) as usize,
        ));
        mandelbrot::draw_set(canvas, set).map_err(|err| err.to_string()).unwrap();

        self // FIXME *
    }
}

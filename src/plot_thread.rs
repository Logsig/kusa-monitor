use crate::plotters::mandelbrot;
use crate::utils::create_mt;
use crate::plotters::Chart;

use plotters::prelude::*;
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
    pub async fn mandelbrot(self, canvas: HtmlCanvasElement) -> js_sys::Array {
        let root = mandelbrot::create_root(canvas);
        let chart = mandelbrot::create_chart(&root).unwrap();

        // self.compute_and_draw_obsolete(&root, &chart).await;
        //====
        self.compute_and_draw(&root, &chart).await;

        js_sys::Array::of2(
            &JsValue::from(self), // FIXME *
            &JsValue::from(Chart::from_ctx(chart)))
    }

    async fn compute_and_draw<'a>(
        &self,
        root: &'a DrawingArea<CanvasBackend, plotters::coord::Shift>,
        chart: &ChartContext<'a, CanvasBackend, RangedCoord<RangedCoordf64, RangedCoordf64>>,
    ) {
        let (real, complex, samples, offset) = mandelbrot::get_params(chart);
        let max_iter = 100;
        let salt = 7;

        // let jsv = exec!(self.th, move || {
        //     let arr = mandelbrot::mandelbrot_data_image(
        //         real, complex, samples, max_iter, salt);
        //
        //     // TODO transferables !!
        //     Ok(JsValue::from(arr))
        // }).await.unwrap();

        // let wh = (samples.0 as u32, samples.1 as u32);
        // mandelbrot::draw_set_as_image(
        //     &root, &ctx, wh, offset, set, max_iter, 5).unwrap();


    }

    #[allow(dead_code)]
    async fn compute_and_draw_obsolete<'a>(
        &self,
        root: &'a DrawingArea<CanvasBackend, plotters::coord::Shift>,
        chart: &ChartContext<'a, CanvasBackend, RangedCoord<RangedCoordf64, RangedCoordf64>>,
    ) {
        let (real, complex, samples, offset) = mandelbrot::get_params(chart);
        let max_iter = 100;
        let salt = 5;

        let jsv = exec!(self.th, move || {
            let arr = mandelbrot::mandelbrot_data_raw(real, complex, samples, max_iter);

            // TODO transferables !!
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

        mandelbrot::draw_set(root, chart, set, max_iter, salt)
            .map_err(|err| err.to_string()).unwrap();
    }
}

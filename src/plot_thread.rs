use crate::plotters::mandelbrot;
use crate::utils::create_mt;
use crate::plotters::Chart;

use plotters::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use js_sys::{ArrayBuffer, Float64Array, Uint8Array, Uint32Array, Object};
use wasm_mt::prelude::*;
use wasm_mt::utils::console_ln;

use std::cell::RefCell;

#[wasm_bindgen]
pub struct PlotThread {
    mt: wasm_mt::WasmMt,
    th: RefCell<Option<wasm_mt::Thread>>,
}

macro_rules! th { ($self:expr) => ($self.th.borrow().as_ref().unwrap()); }

#[wasm_bindgen]
impl PlotThread {
    #[wasm_bindgen(constructor)]
    pub fn new(clazz: &Object) -> Self {
        console_ln!("PlotThread::new(): hi");
        Self {
            mt: create_mt(clazz),
            th: RefCell::new(None),
        }
    }

    pub async fn and_init(self) -> Self {
        self.mt.init().await.unwrap();

        let th = self.mt.thread().and_init().await.unwrap();
        self.th.replace(Some(th));

        self
    }

    // FIXME * -- https://github.com/rustwasm/wasm-bindgen/issues/1858#issuecomment-552108855
    //   pub async fn mandelbrot(&self) { ... }
    pub async fn mandelbrot(self, element: HtmlCanvasElement, max_iter: usize) -> Result<js_sys::Array, JsValue> {
        let ctx = element.get_context("2d")?.unwrap().dyn_into::<CanvasRenderingContext2d>()?;
        let root = mandelbrot::create_root(element);
        let chart = mandelbrot::create_chart(&root).unwrap();

        let perf = web_sys::window().unwrap().performance().unwrap();
        let time_start = perf.now();

        // self.draw_obsolete(&root, &chart, max_iter).await?;
        //====
        self.draw(&chart, &ctx, max_iter).await?;

        console_ln!("@@ `draw` took {:.2}ms", perf.now() - time_start);

        Ok(js_sys::Array::of2(
            &JsValue::from(self), // FIXME *
            &JsValue::from(Chart::from_ctx(chart))))
    }

    async fn draw<'a>(
        &self,
        chart: &ChartContext<'a, CanvasBackend, RangedCoord<RangedCoordf64, RangedCoordf64>>,
        ctx: &CanvasRenderingContext2d,
        max_iter: usize,
    ) -> Result<(), JsValue> {
        let (real, complex, samples, offset) = mandelbrot::get_params(chart);
        let jsv = exec!(th!(self), move || {
            let ab = mandelbrot::mandelbrot_data_image(
                real, complex, samples, max_iter, 5);

            // TODO transferables !!
            Ok(JsValue::from(ab))
        }).await?;

        let (width, height) = (samples.0 as u32, samples.1 as u32);
        let arr = Uint8Array::new(jsv.dyn_ref::<ArrayBuffer>().unwrap());

        // TODO: Instead of `arr.to_vec()`, maybe use `arr.copy_to()`
        //   with pre-allocated `Vec<u8>` in `self`
        // https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.Uint8Array.html#method.copy_to
        let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&mut arr.to_vec()), width, height)?.into();

        ctx.put_image_data(&data, offset.0 as f64, offset.1 as f64)?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn draw_obsolete<'a>(
        &self,
        root: &'a DrawingArea<CanvasBackend, plotters::coord::Shift>,
        chart: &ChartContext<'a, CanvasBackend, RangedCoord<RangedCoordf64, RangedCoordf64>>,
        max_iter: usize,
    ) -> Result<(), JsValue> {
        let (real, complex, samples, _offset) = mandelbrot::get_params(chart);
        let jsv = exec!(th!(self), move || {
            let arr = mandelbrot::mandelbrot_data_raw(
                real, complex, samples, max_iter);

            // TODO transferables !!
            Ok(JsValue::from(arr))
        }).await?;

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

        mandelbrot::draw_set(root, chart, set, max_iter, 5)
            .map_err(|err| err.to_string()).unwrap();

        Ok(())
    }
}

#![feature(async_closure)]

mod utils;
use crate::utils::create_mt;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use js_sys::{Object, Reflect};
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, Counter, sleep};

#[wasm_bindgen]
pub fn app(clazz: Object) -> Result<(), JsValue> {
    spawn_local(async move {
        let _ = entry_main().await;
    });

    let mt = create_mt(&clazz);
    spawn_local(async move {
        let th = mt.thread().and_init().await.unwrap();

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

//=========== Prototype `PlotThread`
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

//=========== plotters stuff
mod plotters;
use crate::plotters::{func_plot, mandelbrot};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Type alias for the result of a drawing function.
pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Type used on the JS side to convert screen coordinates to chart
/// coordinates.
#[wasm_bindgen]
pub struct Chart {
    convert: Box<dyn Fn((i32, i32)) -> Option<(f64, f64)>>,
}

/// Result of screen to chart coordinates conversion.
#[wasm_bindgen]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl Chart {
    /// Draw provided power function on the canvas element using it's id.
    /// Return `Chart` struct suitable for coordinate conversion.
    pub fn power(canvas_id: &str, power: i32) -> Result<Chart, JsValue> {
        let map_coord = func_plot::draw(canvas_id, power).map_err(|err| err.to_string())?;
        Ok(Chart {
            convert: Box::new(move |coord| map_coord(coord).map(|(x, y)| (x.into(), y.into()))),
        })
    }

    /// Draw Mandelbrot set on the provided canvas element.
    /// Return `Chart` struct suitable for coordinate conversion.
    pub fn mandelbrot(canvas: HtmlCanvasElement) -> Result<Chart, JsValue> {
        let map_coord = mandelbrot::draw(canvas).map_err(|err| err.to_string())?;
        Ok(Chart {
            convert: Box::new(map_coord),
        })
    }

    /// This function can be used to convert screen coordinates to
    /// chart coordinates.
    pub fn coord(&self, x: i32, y: i32) -> Option<Point> {
        (self.convert)((x, y)).map(|(x, y)| Point { x, y })
    }
}

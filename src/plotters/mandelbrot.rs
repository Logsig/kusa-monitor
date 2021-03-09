use crate::DrawResult;
use plotters::prelude::*;
use std::ops::Range;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use js_sys::{Float64Array, Uint32Array};
use wasm_mt::utils::console_ln;

pub fn create_root(element: HtmlCanvasElement)
-> DrawingArea<plotters::drawing::CanvasBackend, plotters::coord::Shift> {
    let backend = CanvasBackend::with_canvas_object(element).unwrap();

    let root = backend.into_drawing_area();
    root
}

pub fn create_chart(root: &DrawingArea<plotters::drawing::CanvasBackend, plotters::coord::Shift>)
-> Result<ChartContext<plotters::drawing::CanvasBackend, RangedCoord<RangedCoordf64, RangedCoordf64>>, Box<dyn std::error::Error>> {
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_ranged(-2.1..0.6, -1.2..1.2)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    Ok(chart)
}

pub fn get_params(chart: &ChartContext<'_, plotters::drawing::CanvasBackend, RangedCoord<RangedCoordf64, RangedCoordf64>>)
-> (Range<f64>, Range<f64>, (i32, i32), (i32, i32)) {
    let plotting_area = chart.plotting_area();

    let range = plotting_area.get_pixel_range();
    let (pw, ph) = (range.0.end - range.0.start, range.1.end - range.1.start);
    let (xr, yr) = (chart.x_range(), chart.y_range());

    (xr, yr, (pw, ph), (range.0.start, range.1.start))
}

/// Draw Mandelbrot set
pub fn draw(element: HtmlCanvasElement)
-> DrawResult<impl Fn((i32, i32)) -> Option<(f64, f64)>> {
    let ctx = element.get_context("2d").unwrap().unwrap().dyn_into().unwrap();
    let root = create_root(element);

    let chart = create_chart(&root).unwrap();
    let (real, complex, samples, offset) = get_params(&chart);

    let perf = web_sys::window().unwrap().performance().unwrap();

    let time_start = perf.now();
    let max_iter = 100;
    let set = mandelbrot_set(real, complex, samples, max_iter);
    // console_ln!("@@ Took {:.2}ms", perf.now() - time_start); // ~ 0; just creating an iterator

    let time_start = perf.now();
    // draw_set(&root, &chart, set, max_iter, 0).unwrap(); // slow!!
    //====
    let wh = (samples.0 as u32, samples.1 as u32);
    draw_set_via_image(&root, &ctx, wh, offset, set, max_iter, 0).unwrap();
    console_ln!("@@ Drawing took {:.2}ms", perf.now() - time_start);

    Ok(Box::new(chart.into_coord_trans()))
}

pub fn draw_set<'a>(
    root: &'a DrawingArea<plotters::drawing::CanvasBackend, plotters::coord::Shift>,
    chart: &ChartContext<'a, plotters::drawing::CanvasBackend, RangedCoord<RangedCoordf64, RangedCoordf64>>,
    set: impl Iterator<Item = (f64, f64, usize)>,
    max_iter: usize,
    salt: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let plotting_area = chart.plotting_area();

    for (x, y, c) in set {
        if c < max_iter {
            plotting_area.draw_pixel((x, y), &HSLColor((c + salt as usize) as f64 / 100.0, 1.0, 0.5))?;
        } else {
            plotting_area.draw_pixel((x, y), &BLACK)?;
        }
    }

    root.present().unwrap();
    Ok(())
}

pub fn draw_set_via_image(
    root: &DrawingArea<plotters::drawing::CanvasBackend, plotters::coord::Shift>,
    ctx: &CanvasRenderingContext2d,
    wh: (u32, u32),
    offset: (i32, i32),
    set: impl Iterator<Item = (f64, f64, usize)>,
    max_iter: usize,
    salt: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = wh;

    //======== TODO calc as arraybuffer; deprecate `mandelbrot_arr_ab()`
    let mut data = vec![255; (4 * width * height) as usize];
    for (idx, (_x, _y, c)) in set.enumerate() {
        let (r, g, b) = if c < max_iter {
            plotters::style::Color::rgb(
                &HSLColor((c + salt as usize) as f64 / 100.0, 1.0, 0.5))
        } else {
            (0, 0, 0) // black
        };

        data[4 * idx] = r;
        data[4 * idx + 1] = g;
        data[4 * idx + 2] = b;
        // data[4 * idx + 3] = 255;
    }
    //========

    let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&mut data), width, height).unwrap().into();
    ctx.put_image_data(&data, offset.0 as f64, offset.1 as f64).unwrap();

    root.present().unwrap();
    Ok(())
}

pub fn mandelbrot_arr_ab(
    real: Range<f64>,
    complex: Range<f64>,
    samples: (i32, i32),
    max_iter: usize,
) -> js_sys::Array {
    let arr_x = js_sys::Array::new();
    let arr_y = js_sys::Array::new();
    let arr_c = js_sys::Array::new();

    // for (x, y, c) in [
    //     (-0.5 as f64, 0.5 as f64, 100 as usize),
    //     (-1.0 as f64, 0.5 as f64, 100 as usize),
    // ].iter().map(|&p| p) { // https://stackoverflow.com/questions/30467085/how-to-iterate-over-and-filter-an-array
    //====
    for (x, y, c) in mandelbrot_set(real, complex, samples, max_iter) {
        arr_x.push(&JsValue::from(x));
        arr_y.push(&JsValue::from(y));
        arr_c.push(&JsValue::from(c as u32));
    }

    js_sys::Array::of3(
        &Float64Array::new(&arr_x).buffer().into(),
        &Float64Array::new(&arr_y).buffer().into(),
        &Uint32Array::new(&arr_c).buffer().into())
}

fn mandelbrot_set(
    real: Range<f64>,
    complex: Range<f64>,
    samples: (i32, i32),
    max_iter: usize,
) -> impl Iterator<Item = (f64, f64, usize)> {
    let step = (
        (real.end - real.start) / samples.0 as f64,
        (complex.end - complex.start) / samples.1 as f64,
    );

    let samples = (samples.0 as usize, samples.1 as usize);
    (0..(samples.0 * samples.1)).map(move |k| {
        let c = (
            real.start + step.0 * (k % samples.0) as f64,
            complex.start + step.1 * (k / samples.0) as f64,
        );
        let mut z = (0.0, 0.0);
        let mut cnt = 0;
        while cnt < max_iter && z.0 * z.0 + z.1 * z.1 <= 1e10 {
            z = (z.0 * z.0 - z.1 * z.1 + c.0, 2.0 * z.0 * z.1 + c.1);
            cnt += 1;
        }
        (c.0, c.1, cnt)
    })
}

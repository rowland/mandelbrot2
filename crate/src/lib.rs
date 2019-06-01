use std::cmp;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, ImageData};
use js_sys::{Date};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Params {
    width: i32,
    height: i32,
    x: f64,
    y: f64,
    mag: f64,
    limit: i32,
}

fn params(hash: String) -> Params {
    let hash = hash.trim_start_matches("#");
    let parsed_url = url::form_urlencoded::parse(hash.as_bytes());
    let mut hash_query: HashMap<String, String> = parsed_url.into_owned().collect();
    Params {
        width: hash_query
            .entry("w".to_string())
            .or_default()
            .parse()
            .unwrap_or(800),
        height: hash_query
            .entry("h".to_string())
            .or_default()
            .parse()
            .unwrap_or(600),
        x: hash_query
            .entry("x".to_string())
            .or_default()
            .parse()
            .unwrap_or(-0.5),
        y: hash_query
            .entry("y".to_string())
            .or_default()
            .parse()
            .unwrap_or(0.0),
        mag: hash_query
            .entry("mag".to_string())
            .or_default()
            .parse()
            .unwrap_or(1.5),
        limit: hash_query
            .entry("limit".to_string())
            .or_default()
            .parse()
            .unwrap_or(1000),
    }
}

type Complex = (f64, f64);

fn iter_area(w: i32, h: i32, center: Complex, mag: f64, iter_callback: &mut FnMut(i32, Complex)) {
    let (cx, cy) = center;
    let p = 4.0 / (cmp::min(w, h) as f64) / mag;
    let x0 = cx - (w as f64) / 2.0 * p;
    let y0 = cy + (h as f64) / 2.0 * p;
    let mut i = 0;
    for y in 0..h {
        for x in 0..w {
            iter_callback(i, (x0 + (x as f64) * p, y0 - (y as f64) * p));
            i += 1;
        }
    }
}

pub fn escapes(p: Complex) -> bool {
    let (a, b) = p;
    a * a + b * b > 4.0
}

pub fn next(p: Complex, p0: Complex) -> Complex {
    let (a, b) = p;
    let (a0, b0) = p0;
    (a * a - b * b + a0, 2.0 * a * b + b0)
}

pub fn iterations(mut p: Complex, limit: i32) -> i32 {
    let p0 = p;
    for i in 0..(limit + 1) {
        if escapes(p) {
            return i;
        }
        p = next(p, p0);
    }
    return limit;
}

type Color = [u8; 4];

static PALETTE: [Color; 16] = [
    [66, 30, 15, 255],
    [25, 7, 26, 255],
    [9, 1, 47, 255],
    [4, 4, 73, 255],
    [0, 7, 100, 255],
    [12, 44, 138, 255],
    [24, 82, 177, 255],
    [57, 125, 209, 255],
    [134, 181, 229, 255],
    [211, 236, 248, 255],
    [241, 233, 191, 255],
    [248, 201, 95, 255],
    [255, 170, 0, 255],
    [204, 128, 0, 255],
    [153, 87, 0, 255],
    [106, 52, 3, 255],
];
static BLACK: Color = [0, 0, 0, 255];

pub fn color(v: i32, limit: i32) -> Color {
    if v > 0 && v < limit {
        return PALETTE[(v as usize) % 16];
    }
    return BLACK;
}

fn draw_mandelbrot_set(params: &Params, img: &mut Vec<u8>) {
    iter_area(
        params.width,
        params.height,
        (params.x, params.y),
        params.mag,
        &mut |i: i32, p: Complex| {
            let v = iterations(p, params.limit);
            let pixel = color(v, params.limit);
            let offset = (i * 4) as usize;
            for b in 0..4 {
                img[offset + b] = pixel[b];
            }
        },
    );
}

// Called by our JS entry point to run the example.
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let window = web_sys::window().expect("should have a Window");
    let document = window.document().expect("should have a Document");
    let location = document.location().expect("should have a Location");
    let body = document.body().expect("should have a body");
    let hash = location.hash().unwrap();
    let p = params(hash);
    let title = format!(
        "Mandelbrot2 x={}, y={}, mag={}, limit={}",
        p.x, p.y, p.mag, p.limit
    );

    let p_title: web_sys::Node = document.create_element("p")?.into();
    p_title.set_text_content(Some(&title));

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.style().set_property("border", "solid green 1px")?;
    canvas.set_width(p.width as u32);
    canvas.set_height(p.height as u32);


    body.append_child(&p_title)?;
    body.append_child(&canvas)?;

    let size = p.width * p.height * 4;
    let mut img = vec![0u8; size as usize];
    let t1 = Date::now();
    draw_mandelbrot_set(&p, &mut img);
    let t2 = Date::now();
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut img), p.width as u32, p.height as u32)?;

    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    ctx.put_image_data(&data, 0.0, 0.0)?;
    let p_now: web_sys::Node = document.create_element("p")?.into();
    let elapsed = format!("elapsed: {}",  t2 - t1);

    p_now.set_text_content(Some(&elapsed));
    body.append_child(&p_now)?;

    Ok(())
}

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

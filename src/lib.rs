mod chip8emulator;

use chip8emulator::Chip8Emulator;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use wasm_bindgen::JsCast;
use js_sys::Uint8Array;
use gloo::{events::EventListener};
use std::rc::Rc;
use std::cell::RefCell;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ROMS_DIR: &str = "roms";
const PIXEL_OFF_COLOR: &str = "#000000";
const PIXEL_ON_COLOR: &str = "#00a86b";

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub async fn main_js() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

    let mut chip8 =
        Rc::new(RefCell::new(Chip8Emulator::new(get_current_time())));

    set_canvas_size(chip8.borrow().get_gfx_width(), chip8.borrow().get_gfx_height());

    let rom_name = "15PUZZLE";
    let path = format!("{}/{}", ROMS_DIR, rom_name);

    let buffer = get_binary_file(&path).await
        .expect(&format!("Can't load {}", path));

    chip8.borrow_mut().load_rom(&buffer);

    register_inputs(&chip8);

    if chip8.borrow_mut().gfx_needs_rerender() {
        render(&chip8);
    }
}

fn set_canvas_size(width: u32, height: u32) {
    let canvas = get_context().canvas().unwrap();
    canvas.set_width(width);
    canvas.set_height(height);
}

fn render(chip8: &Rc<RefCell<chip8emulator::Chip8Emulator>>) {
    let chip8 = chip8.borrow_mut();
    let width = chip8.get_gfx_width();
    let height = chip8.get_gfx_height();

    let ctx = get_context();
    ctx.begin_path();

    ctx.set_fill_style(&PIXEL_OFF_COLOR.into());
    ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

    ctx.set_fill_style(&PIXEL_ON_COLOR.into());
    for x in 0..width {
        for y in 0..height {
            if chip8.get_gfx_pixel(x, y) {
                ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
            }
        }
    }

    ctx.stroke();
}

async fn get_binary_file(path: &str) -> Result<Vec<u8>, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(path)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    if !resp.ok() {
        return Err(JsValue::NULL);
    }

    let buffer = JsFuture::from(resp.array_buffer()?).await?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

fn register_inputs(chip8: &Rc<RefCell<chip8emulator::Chip8Emulator>>) {
    let chip8_tmp = Rc::clone(&chip8);
    EventListener::new(&web_sys::window().unwrap(), "keydown", move |e| {
        let e: web_sys::KeyboardEvent = e.clone().dyn_into().unwrap();
        if let Some(key) = jskey_to_chip8key(&e.key()) {
            web_sys::console::log_1(&format!("Key down: {:X}", key).into());
            chip8_tmp.borrow_mut().keydown(key);
        }
    }).forget();

    let chip8_tmp = Rc::clone(&chip8);
    EventListener::new(&web_sys::window().unwrap(), "keyup", move |e| {
        let e: web_sys::KeyboardEvent = e.clone().dyn_into().unwrap();
        if let Some(key) = jskey_to_chip8key(&e.key()) {
            web_sys::console::log_1(&format!("Key up: {:X}", key).into());
            chip8_tmp.borrow_mut().keyup(key);
        }
    }).forget();
}

fn jskey_to_chip8key(key: &str) -> Option<u8> {
    match key {
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(0xC),
        "q" => Some(4),
        "w" => Some(5),
        "e" => Some(6),
        "r" => Some(0xD),
        "a" => Some(7),
        "s" => Some(8),
        "d" => Some(9),
        "f" => Some(0xE),
        "z" => Some(0xA),
        "x" => Some(0),
        "c" => Some(0xB),
        "v" => Some(0xF),
        _ => None
    }
}

thread_local! {
    static PERFORMANCE: web_sys::Performance =
        web_sys::window().unwrap().performance().unwrap();

    static CONTEXT: web_sys::CanvasRenderingContext2d =
        web_sys::window().unwrap().document().unwrap()
            .get_element_by_id("canvas").expect("No element with id #canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element with id #canvas is not a canvas")
            .get_context("2d").unwrap().unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
}

fn get_current_time() -> f64 {
    PERFORMANCE.with(|p| p.now())
}

fn get_context() -> web_sys::CanvasRenderingContext2d {
    CONTEXT.with(|c| c.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};
    wasm_bindgen_test_configure!(run_in_browser);
}
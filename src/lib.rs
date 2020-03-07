mod timer;
mod graphics;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use arrayvec::ArrayVec;
use timer::Timer;
use graphics::Graphics;
use wasm_bindgen::JsCast;
use js_sys::Uint8Array;


// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ROMS_DIR: &str = "roms";
const WIDTH: usize = 64;
const HEIGHT: usize = 32;

struct Chip8Emulator {
    memory: [u8; 4096],
    V: [u8; 16],
    I: u16,
    pc: u16,
    gfx: Graphics,
    delay_timer: Timer,
    sound_timer: Timer,
    stack: ArrayVec<[u16; 16]>,
    keypad: [bool; 16],
}

const PROGRAM_MEMORY_START: usize = 0x200;

impl Chip8Emulator {
    fn new() -> Chip8Emulator {
        Chip8Emulator {
            memory: [0; 4096],
            V: [0; 16],
            I: 0,
            pc: PROGRAM_MEMORY_START as u16,
            gfx: Graphics::new(),
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
            stack: ArrayVec::new(),
            keypad: [false; 16],
        }
    }

    fn load_rom(&mut self, rom_data: &[u8]) {
        let end_index = PROGRAM_MEMORY_START + rom_data.len();
        self.memory[PROGRAM_MEMORY_START..end_index].clone_from_slice(rom_data);
    }

    fn reset(&mut self) {
        *self = Chip8Emulator::new();
    }

    fn render(&self) {
        let ctx = get_context();

        let canvas = ctx.canvas().unwrap();
        canvas.set_width(64);
        canvas.set_height(32);

        ctx.begin_path();

        ctx.set_fill_style(&"#000000".into());
        ctx.fill_rect(0.0, 0.0, WIDTH as f64, HEIGHT as f64);

        ctx.set_fill_style(&"#00a86b".into());
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if self.gfx.get(x, y) {
                    ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
                }
            }
        }

        ctx.stroke();
    }
}

thread_local! {
static _context: web_sys::CanvasRenderingContext2d =
    web_sys::window().unwrap().document().unwrap()
        .get_element_by_id("canvas").unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap()
        .get_context("2d") .unwrap() .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>() .unwrap()
}

fn get_context() -> web_sys::CanvasRenderingContext2d {
    _context.with(|c| c.clone().clone())
}

async fn get_binary_file(path: &str) -> Result<Vec<u8>, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(path)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let buffer = JsFuture::from(resp.array_buffer()?).await?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub async fn main_js() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

    let mut chip8 = Chip8Emulator::new();

    let path = format!("{}/15PUZZLE", ROMS_DIR);
    let buffer = get_binary_file(&path).await
        .expect(&format!("Can't load {}", path));

    chip8.load_rom(&buffer);
    chip8.gfx.toggle(1, 1);
    chip8.render();
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_load_rom() {
        let mut chip8 = Chip8Emulator::new();
        let data = [1u8, 5, 3, 5, 1, 255, 9];
        chip8.load_rom(&data);

        for i in 0..data.len() {
            assert_eq!(chip8.memory[PROGRAM_MEMORY_START + i], data[i]);
        }

        for i in 0..5 {
            assert_eq!(chip8.memory[PROGRAM_MEMORY_START - i - 1], 0);
            assert_eq!(chip8.memory[PROGRAM_MEMORY_START + data.len() + i], 0);
        }
    }

    #[wasm_bindgen_test]
    fn test_reset() {
        let mut chip8 = Chip8Emulator::new();
        let data = [1u8, 5, 3, 5, 1, 255, 9];
        chip8.load_rom(&data);
        chip8.reset();
        for i in 0..data.len() {
            assert_eq!(chip8.memory[PROGRAM_MEMORY_START + i], 0);
        }
    }
}
mod timer;
mod graphics;

use wasm_bindgen::prelude::*;
use web_sys::console;
use arrayvec::ArrayVec;
use timer::Timer;
use graphics::Graphics;


// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
pub struct Chip8Emulator {
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

#[wasm_bindgen]
impl Chip8Emulator {
    pub fn new() -> Chip8Emulator {
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

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        let end_index = PROGRAM_MEMORY_START + rom_data.len();
        self.memory[PROGRAM_MEMORY_START..end_index].clone_from_slice(rom_data);
    }

    pub fn reset(&mut self) {
        *self = Chip8Emulator::new();
    }
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

    // Your code goes here!
    console::log_1(&"Hello world!".into());
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
}
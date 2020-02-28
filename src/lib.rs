use wasm_bindgen::prelude::*;
use web_sys::console;
use fixedbitset::FixedBitSet;

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
    I: MemoryAddress,
    pc: MemoryAddress,
    gfx: Graphics,
}

#[derive(Copy, Clone)]
struct MemoryAddress(u16);

impl MemoryAddress {
    fn new(addr: u16) -> MemoryAddress {
        if addr > 0x0FFF {
            panic!("Memory address {:X} out of bounds (0x000 to 0xFFF)", addr);
        }

        MemoryAddress(addr)
    }

    fn value(&self) -> u16 {
        self.0
    }
}

struct Graphics {
    width: usize,
    height: usize,
    display: FixedBitSet,
}

impl Graphics {
    fn new() -> Graphics {
        let width = 64;
        let height = 32;
        let display = FixedBitSet::with_capacity(width * height);
        Graphics { width, height, display }
    }

    /// Toggles the pixel at column `x` and row `y` (0-indexed) on the display
    /// and returns whether a pixel was toggled from on to off.
    fn toggle(&mut self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            panic!("Pixel ({}, {}) is out of bounds of display size {}x{}",
                   x, y, self.width, self.height);
        }
        let index = y * self.width + x;
        let res = self.display[index];
        self.display.toggle(index);
        res
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
    use crate::*;

    #[test]
    fn test_graphics_toggle() {
        use fixedbitset::FixedBitSet;
        let mut gfx = Graphics {
            width: 2,
            height: 2,
            display: FixedBitSet::with_capacity(4),
        };

        gfx.display.insert(2);
        assert_eq!(gfx.toggle(0, 1), true);
        assert_eq!(gfx.toggle(0, 1), false);
        assert!(gfx.display[2]);
        assert_eq!(gfx.toggle(0, 1), true);
        assert_eq!(gfx.toggle(1, 1), false);
        assert!(gfx.display[3]);
        assert_eq!(gfx.toggle(0, 0), false);
        assert_eq!(gfx.toggle(1, 1), true);
        assert!(gfx.display[0] && !gfx.display[1] &&
            !gfx.display[2] && !gfx.display[3]);
    }
}
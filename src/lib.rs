use wasm_bindgen::prelude::*;
use web_sys::console;
use fixedbitset::FixedBitSet;
use arrayvec::ArrayVec;
use core::ops::AddAssign;

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
    delay_timer: Timer,
    sound_timer: Timer,
    stack: ArrayVec<[MemoryAddress; 16]>,
    keypad: [u8; 16],
}

impl Chip8Emulator {}

#[derive(Copy, Clone)]
struct MemoryAddress(u16);

impl MemoryAddress {
    fn new(addr: u16) -> MemoryAddress {
        assert!(addr <= 0x0FFF,
                "Memory address {:X} out of bounds (0x000 to 0xFFF)", addr);

        MemoryAddress(addr)
    }

    fn value(&self) -> u16 {
        self.0
    }
}

impl AddAssign<u16> for MemoryAddress {
    fn add_assign(&mut self, rhs: u16) {
        *self = MemoryAddress::new(self.0 + rhs)
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

    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }
}


struct Timer {
    value: u8,
    next_time: f64,
    performance: web_sys::Performance,
}

impl Timer {
    fn new() -> Timer {
        let performance = web_sys::window().unwrap().performance().unwrap();
        Timer {
            value: 0,
            next_time: performance.now(),
            performance,
        }
    }

    fn step(&mut self) {
        if self.performance.now() > self.next_time {
            self.value = self.value.saturating_sub(1);
            self.next_time += 1000.0 / 60.0;
        }
    }

    fn value(&self) -> u8 {
        self.value
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
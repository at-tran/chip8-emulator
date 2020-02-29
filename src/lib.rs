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
    prev_time: f64,
}

impl Timer {
    const INTERVAL: f64 = 1000.0 / 60.0;

    fn new() -> Timer {
        Timer {
            value: 0,
            prev_time: now(),
        }
    }

    fn step(&mut self) {
        let ticks = (now() - self.prev_time) / Timer::INTERVAL;
        assert!(ticks >= 0.0);
        self.value = self.value.saturating_sub(ticks as u8);
        self.prev_time += ticks.floor() * Timer::INTERVAL;
    }

    fn value(&self) -> u8 {
        self.value
    }

    fn set_value(&mut self, value: u8) {
        self.value = value;
    }
}

thread_local! {
    static performance: web_sys::Performance =
        web_sys::window().unwrap().performance().unwrap();
}

fn now() -> f64 {
    performance.with(|p| { p.now() })
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

    use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};
    wasm_bindgen_test_configure!(run_in_browser);

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

    #[wasm_bindgen_test]
    fn test_timer() {
        let mut timer = Timer::new();
        timer.set_value(5);

        let t = now();
        // Add 2ms to give enough for timer.step()
        while now() + 2.0 - t < Timer::INTERVAL { timer.step(); }
        assert_eq!(timer.value(), 5);
        while now() + 2.0 - t < 2.0 * Timer::INTERVAL { timer.step(); }
        assert_eq!(timer.value(), 4);

        let t = now();
        while now() - t < 2.0 * Timer::INTERVAL {}
        timer.step();
        assert_eq!(timer.value(), 2);
    }
}
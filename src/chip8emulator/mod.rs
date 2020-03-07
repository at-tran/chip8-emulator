mod timer;
mod graphics;

use timer::Timer;
use graphics::Graphics;
use arrayvec::ArrayVec;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const PROGRAM_MEMORY_START: usize = 0x200;

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

impl Chip8Emulator {
    pub fn new(current_time: f64) -> Chip8Emulator {
        Chip8Emulator {
            memory: [0; 4096],
            V: [0; 16],
            I: 0,
            pc: PROGRAM_MEMORY_START as u16,
            gfx: Graphics::new(WIDTH, HEIGHT),
            delay_timer: Timer::new(current_time),
            sound_timer: Timer::new(current_time),
            stack: ArrayVec::new(),
            keypad: [false; 16],
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        let end_index = PROGRAM_MEMORY_START + rom_data.len();
        self.memory[PROGRAM_MEMORY_START..end_index].clone_from_slice(rom_data);
    }

    pub fn reset(&mut self, current_time: f64) {
        *self = Chip8Emulator::new(current_time);
    }

    pub fn get_gfx_width(&self) -> u32 {
        self.gfx.get_width()
    }

    pub fn get_gfx_height(&self) -> u32 {
        self.gfx.get_height()
    }

    pub fn get_gfx_pixel(&self, x: u32, y: u32) -> bool {
        self.gfx.get_pixel(x, y)
    }

    pub fn gfx_needs_rerender(&mut self) -> bool {
        self.gfx.needs_rerender()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rom() {
        let mut chip8 = Chip8Emulator::new(0.0);
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

    #[test]
    fn test_reset() {
        let mut chip8 = Chip8Emulator::new(0.0);
        let data = [1u8, 5, 3, 5, 1, 255, 9];
        chip8.load_rom(&data);
        chip8.reset(1.0);
        for i in 0..data.len() {
            assert_eq!(chip8.memory[PROGRAM_MEMORY_START + i], 0);
        }
    }

    #[test]
    fn test_gfx() {
        let mut chip8 = Chip8Emulator::new(0.0);

        assert_eq!(chip8.get_gfx_width(), WIDTH);
        assert_eq!(chip8.get_gfx_height(), HEIGHT);
        assert!(chip8.gfx_needs_rerender());
        assert!(!chip8.gfx_needs_rerender());
        assert!(!chip8.get_gfx_pixel(5, 5));
        chip8.gfx.toggle(5, 5);
        assert!(chip8.get_gfx_pixel(5, 5));
        assert!(chip8.gfx.toggle(5, 5));
    }
}
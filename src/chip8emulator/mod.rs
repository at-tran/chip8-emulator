mod timer;
mod graphics;
mod keypad;
mod chip8timer;

use timer::Timer;
use graphics::Graphics;
use keypad::KeyPad;
use chip8timer::Chip8Timer;
use arrayvec::ArrayVec;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const PROGRAM_MEMORY_START: usize = 0x200;

#[allow(non_snake_case)]
pub struct Chip8Emulator {
    memory: [u8; 4096],
    V: [u8; 16],
    I: u16,
    pc: u16,
    gfx: Graphics,
    delay_timer: Chip8Timer,
    sound_timer: Chip8Timer,
    stack: ArrayVec<[u16; 16]>,
    keypad: KeyPad,
    timer: Timer,
}

impl Chip8Emulator {
    pub fn new(current_time: f64) -> Chip8Emulator {
        Chip8Emulator {
            memory: [0; 4096],
            V: [0; 16],
            I: 0,
            pc: PROGRAM_MEMORY_START as u16,
            gfx: Graphics::new(WIDTH, HEIGHT),
            delay_timer: Chip8Timer::new(current_time),
            sound_timer: Chip8Timer::new(current_time),
            stack: ArrayVec::new(),
            keypad: KeyPad::new(),
            timer: Timer::new(current_time, 1000.0 / 600.0),
        }
    }

    pub fn tick(&mut self, current_time: f64) {
        for _ in 0..self.timer.step(current_time) as u32 {
            let opcode = self.get_next_opcode();
            // web_sys::console::log_1(&format!("{:X}", opcode).into());
            match get_nibbles(opcode, 0, 1) {
                0 => match get_nibbles(opcode, 1, 4) {
                    0x0e0 => self.clear_screen(),
                    0x0ee => self.return_subroutine(),
                    address => self.execute_subroutine(address)
                }
                1 => self.jump_to(get_nibbles(opcode, 1, 4)),
                2 => self.execute_subroutine(get_nibbles(opcode, 1, 4)),
                3 => self.skip_if_eq(get_nibbles(opcode, 1, 2) as u8,
                                     get_nibbles(opcode, 2, 4) as u8),
                4 => self.skip_if_ne(get_nibbles(opcode, 1, 2) as u8,
                                     get_nibbles(opcode, 2, 4) as u8),

                _ => web_sys::console::error_1(&format!("Invalid instruction {:X}", opcode).into())
            }
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

    pub fn keydown(&mut self, key: u8) {
        self.keypad.keydown(key);
    }

    pub fn keyup(&mut self, key: u8) {
        self.keypad.keyup(key);
    }

    pub fn set_ticks_per_second(&mut self, ticks_per_second: f64) {
        self.timer.set_interval(1000.0 / ticks_per_second);
    }

    fn get_next_opcode(&mut self) -> u16 {
        let opcode = ((self.memory[self.pc as usize] as u16) << 8)
            + self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;
        opcode
    }

    fn clear_screen(&mut self) {
        self.gfx.clear();
    }

    fn return_subroutine(&mut self) {
        self.pc = self.stack.pop().expect("Cannot pop empty stack");
    }

    fn execute_subroutine(&mut self, address: u16) {
        self.stack.push(self.pc);
        self.jump_to(address);
    }

    fn jump_to(&mut self, address: u16) {
        self.pc = address;
    }

    fn skip_if_eq(&mut self, v: u8, value: u8) {
        if self.V[v as usize] == value {
            self.pc += 2;
        }
    }

    fn skip_if_ne(&mut self, v: u8, value: u8) {
        if self.V[v as usize] != value {
            self.pc += 2;
        }
    }
}

fn get_nibbles(value: u16, start_index: u16, end_index: u16) -> u16 {
    assert!(0 <= start_index);
    assert!(start_index < end_index);
    assert!(end_index <= 4);

    let mut mask = 0;
    for i in start_index..end_index {
        mask += 0xf << (4 - i);
    };
    (value & mask) >> (4 - end_index)
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

    #[test]
    fn test_get_nibbles() {
        assert_eq!(get_nibbles(0x3a7b, 1, 3), 0xa7);
        assert_eq!(get_nibbles(0x3a7b, 2, 3), 0x7);
        assert_eq!(get_nibbles(0x3a7b, 2, 4), 0x7b);
        assert_eq!(get_nibbles(0x3a7b, 0, 4), 0x3a7b);
    }
}
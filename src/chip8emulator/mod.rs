mod chip8timer;
mod graphics;
mod keypad;
mod timer;

use arrayvec::ArrayVec;
use chip8timer::Chip8Timer;
use graphics::Graphics;
use keypad::KeyPad;
use timer::Timer;

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
            self.execute_next_instruction();
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

    fn execute_next_instruction(&mut self) {
        let opcode = self.get_next_opcode();
        // web_sys::console::log_1(&format!("{:X}", opcode).into());

        match get_nibble(opcode, 0) {
            0 => match get_nibbles(opcode, 1, 4) {
                0x0e0 => self.clear_screen(),
                0x0ee => self.return_subroutine(),
                address => self.execute_subroutine(address),
            },
            1 => self.jump_to(get_nibbles(opcode, 1, 4)),
            2 => self.execute_subroutine(get_nibbles(opcode, 1, 4)),
            3 => self.skip_if_eq(get_nibble(opcode, 1), get_nibbles(opcode, 2, 4) as u8),
            4 => self.skip_if_ne(get_nibble(opcode, 1), get_nibbles(opcode, 2, 4) as u8),
            5 => match get_nibble(opcode, 3) {
                0 => self.skip_if_eq_reg(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                _ => Chip8Emulator::invalid_instruction(opcode),
            },
            6 => self.store(get_nibble(opcode, 1), get_nibbles(opcode, 2, 4) as u8),
            7 => self.add(get_nibble(opcode, 1), get_nibbles(opcode, 2, 4) as u8),
            8 => match get_nibble(opcode, 3) {
                0 => self.store_reg(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                1 => self.store_reg_or(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                2 => self.store_reg_and(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                3 => self.store_reg_xor(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                4 => self.add_reg(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                5 => self.sub_reg(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                6 => self.store_reg_shr1(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                7 => self.store_reg_sub(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                0xe => self.store_reg_shl1(get_nibble(opcode, 1), get_nibble(opcode, 2)),
                _ => Chip8Emulator::invalid_instruction(opcode),
            },

            _ => Chip8Emulator::invalid_instruction(opcode),
        }
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

    fn skip_if_eq(&mut self, x: u8, value: u8) {
        if self.V[x as usize] == value {
            self.pc += 2;
        }
    }

    fn skip_if_ne(&mut self, x: u8, value: u8) {
        if self.V[x as usize] != value {
            self.pc += 2;
        }
    }

    fn skip_if_eq_reg(&mut self, x: u8, y: u8) {
        if self.V[x as usize] == self.V[y as usize] {
            self.pc += 2;
        }
    }

    fn store(&mut self, x: u8, val: u8) {
        self.V[x as usize] = val;
    }

    fn add(&mut self, x: u8, y: u8) {
        self.V[x as usize] += y;
    }

    fn store_reg(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[y as usize]
    }

    fn store_reg_or(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[x as usize] | self.V[y as usize]
    }

    fn store_reg_and(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[x as usize] & self.V[y as usize]
    }

    fn store_reg_xor(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[x as usize] ^ self.V[y as usize]
    }

    fn add_reg(&mut self, x: u8, y: u8) {
        let (new_val, carry) = self.V[x as usize].overflowing_add(self.V[y as usize]);
        self.V[x as usize] = new_val;
        self.V[0xf] = carry as u8;
    }

    fn sub_reg(&mut self, x: u8, y: u8) {
        let (new_val, borrow) = self.V[x as usize].overflowing_sub(self.V[y as usize]);
        self.V[x as usize] = new_val;
        self.V[0xf] = (!borrow) as u8;
    }

    fn store_reg_shr1(&mut self, x: u8, y: u8) {
        self.V[0xf] = self.V[y as usize] & 0x1;
        self.V[x as usize] = self.V[y as usize] >> 1;
    }

    fn store_reg_sub(&mut self, x: u8, y: u8) {
        let (new_val, borrow) = self.V[y as usize].overflowing_sub(self.V[x as usize]);
        self.V[x as usize] = new_val;
        self.V[0xf] = (!borrow) as u8;
    }

    fn store_reg_shl1(&mut self, x: u8, y: u8) {
        self.V[0xf] = (self.V[y as usize] >> 7) & 0x1;
        self.V[x as usize] = self.V[y as usize] << 1;
    }

    fn invalid_instruction(opcode: u16) {
        web_sys::console::error_1(&format!("Invalid instruction {:X}", opcode).into())
    }
}

fn get_nibble(value: u16, index: u16) -> u8 {
    get_nibbles(value, index, index + 1) as u8
}

fn get_nibbles(value: u16, start_index: u16, end_index: u16) -> u16 {
    assert!(start_index < end_index);
    assert!(end_index <= 4);

    let mut mask = 0;
    for i in start_index..end_index {
        mask += 0xf << (3 - i) * 4;
    }
    (value & mask) >> (4 - end_index) * 4
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
        assert_eq!(get_nibble(0x3a7b, 0), 0x3);
        assert_eq!(get_nibble(0x3a7b, 3), 0xb);
    }

    #[test]
    fn test_get_next_opcode() {
        let mut chip8 = Chip8Emulator::new(0.0);
        let data = [
            0xf1, 0x7d, 0x05, 0x00, 0x13, 0x5c, 0x1a, 0xc4, 0x58, 0xdf, 0x00, 0x01, 0x00, 0x00,
            0x1a, 0x43,
        ];

        let opcodes = [
            0xf17du16, 0x0500, 0x135c, 0x1ac4, 0x58df, 0x0001, 0x0000, 0x1a43,
        ];

        chip8.load_rom(&data);
        for opcode in opcodes.iter() {
            assert_eq!(chip8.get_next_opcode(), *opcode)
        }
    }

    #[test]
    fn test_subroutine() {
        let mut chip8 = Chip8Emulator::new(0.0);
        chip8.jump_to(0xaaaa);
        chip8.execute_subroutine(0x1111);
        assert_eq!(chip8.stack[0], 0xaaaa);
        chip8.return_subroutine();
        assert_eq!(chip8.pc, 0xaaaa);
        assert!(chip8.stack.is_empty())
    }

    #[test]
    fn test_skip() {
        let mut chip8 = Chip8Emulator::new(0.0);
        chip8.jump_to(0);
        chip8.store(0, 5);
        chip8.skip_if_eq(0, 4);
        assert_eq!(chip8.pc, 0);
        chip8.skip_if_eq(0, 5);
        assert_eq!(chip8.pc, 2);
        chip8.skip_if_ne(0, 5);
        assert_eq!(chip8.pc, 2);
        chip8.skip_if_ne(0, 4);
        assert_eq!(chip8.pc, 4);
        chip8.store(1, 5);
        chip8.store(2, 6);
        chip8.skip_if_eq_reg(0, 2);
        assert_eq!(chip8.pc, 4);
        chip8.skip_if_eq_reg(0, 1);
        assert_eq!(chip8.pc, 6);
    }

    #[test]
    fn test_arithmetic() {
        let mut chip8 = Chip8Emulator::new(0.0);

        let x = 0;
        let y = 1;

        chip8.store(x, 10);
        assert_eq!(chip8.V[0], 10);
        chip8.add(x, 5);
        assert_eq!(chip8.V[0], 15);
        chip8.add(x, get_nibbles(0x7005, 2, 4) as u8);
        assert_eq!(chip8.V[0], 20);
        chip8.store(y, 25);
        chip8.store_reg(x, y);
        assert_eq!(chip8.V[x as usize], 25);
        chip8.store_reg(3, x);
        assert_eq!(chip8.V[3], 25);

        chip8.store(x, 10);
        chip8.store_reg_or(x, y);
        assert_eq!(chip8.V[x as usize], 10 | 25);

        chip8.store(x, 10);
        chip8.store_reg_and(x, y);
        assert_eq!(chip8.V[x as usize], 10 & 25);

        chip8.store(x, 10);
        chip8.store_reg_xor(x, y);
        assert_eq!(chip8.V[x as usize], 10 ^ 25);

        chip8.store(x, 254);
        chip8.store(y, 3);
        chip8.add_reg(x, y);
        assert_eq!(chip8.V[x as usize], 1);
        assert_eq!(chip8.V[0xf], 1);
        chip8.add_reg(x, y);
        assert_eq!(chip8.V[x as usize], 4);
        assert_eq!(chip8.V[0xf], 0);
        chip8.sub_reg(x, y);
        assert_eq!(chip8.V[x as usize], 1);
        assert_eq!(chip8.V[0xf], 1);
        chip8.sub_reg(x, y);
        assert_eq!(chip8.V[x as usize], 254);
        assert_eq!(chip8.V[0xf], 0);

        chip8.store(y, 0b10);
        chip8.store_reg_shr1(x, y);
        assert_eq!(chip8.V[x as usize], 1);
        assert_eq!(chip8.V[0xf], 0);

        chip8.store(x, 10);
        chip8.store(y, 5);
        chip8.store_reg_sub(x, y);
        assert_eq!(chip8.V[x as usize], 251);
        assert_eq!(chip8.V[0xf], 0);

        chip8.store(y, 0x80);
        chip8.store_reg_shl1(x, y);
        assert_eq!(chip8.V[x as usize], 0);
        assert_eq!(chip8.V[0xf], 1);
    }
}

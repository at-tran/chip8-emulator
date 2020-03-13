mod chip8timer;
mod graphics;
mod keypad;
mod opcode;
mod timer;

use arrayvec::ArrayVec;
use chip8timer::Chip8Timer;
use graphics::Graphics;
use keypad::KeyPad;
use opcode::Opcode;
use rand;
use timer::Timer;

const WIDTH: u8 = 64;
const HEIGHT: u8 = 32;
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
    waiting_for_keypress: Option<u8>,
    time_needs_updating: bool,
}

impl Chip8Emulator {
    pub fn new(current_time: f64) -> Chip8Emulator {
        Chip8Emulator {
            memory: [0; 4096],
            V: [0; 16],
            I: 0,
            pc: PROGRAM_MEMORY_START as u16,
            gfx: Graphics::new(WIDTH as u32, HEIGHT as u32),
            delay_timer: Chip8Timer::new(current_time),
            sound_timer: Chip8Timer::new(current_time),
            stack: ArrayVec::new(),
            keypad: KeyPad::new(),
            timer: Timer::new(current_time, 1000.0 / 600.0),
            waiting_for_keypress: None,
            time_needs_updating: false,
        }
    }

    pub fn tick(&mut self, current_time: f64) {
        if self.time_needs_updating {
            self.timer.step(current_time);
            self.time_needs_updating = false;
        }

        for _ in 0..self.timer.step(current_time) as u32 {
            if self.waiting_for_keypress.is_none() {
                self.execute_next_instruction();
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
        if let Some(x) = self.waiting_for_keypress {
            self.V[x as usize] = key;
            self.waiting_for_keypress = None;
            self.time_needs_updating = true;
        }
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

        match opcode.get_nibble(0) {
            0 => match opcode.get_nibbles_from(1) {
                0x0e0 => self.clear_screen(),
                0x0ee => self.return_subroutine(),
                address => self.execute_subroutine(address),
            },
            1 => self.jump_to(opcode.get_nibbles_from(1)),
            2 => self.execute_subroutine(opcode.get_nibbles_from(1)),
            3 => self.skip_if_eq(opcode.get_nibble(1), opcode.get_nibbles_from(2) as u8),
            4 => self.skip_if_ne(opcode.get_nibble(1), opcode.get_nibbles_from(2) as u8),
            5 => match opcode.get_nibble(3) {
                0 => self.skip_if_eq_reg(opcode.get_nibble(1), opcode.get_nibble(2)),
                _ => Chip8Emulator::invalid_instruction(opcode),
            },
            6 => self.store(opcode.get_nibble(1), opcode.get_nibbles_from(2) as u8),
            7 => self.add(opcode.get_nibble(1), opcode.get_nibbles_from(2) as u8),
            8 => match opcode.get_nibble(3) {
                0 => self.store_reg(opcode.get_nibble(1), opcode.get_nibble(2)),
                1 => self.store_reg_or(opcode.get_nibble(1), opcode.get_nibble(2)),
                2 => self.store_reg_and(opcode.get_nibble(1), opcode.get_nibble(2)),
                3 => self.store_reg_xor(opcode.get_nibble(1), opcode.get_nibble(2)),
                4 => self.add_reg(opcode.get_nibble(1), opcode.get_nibble(2)),
                5 => self.sub_reg(opcode.get_nibble(1), opcode.get_nibble(2)),
                6 => self.store_reg_shr1(opcode.get_nibble(1), opcode.get_nibble(2)),
                7 => self.store_reg_sub(opcode.get_nibble(1), opcode.get_nibble(2)),
                0xe => self.store_reg_shl1(opcode.get_nibble(1), opcode.get_nibble(2)),
                _ => Chip8Emulator::invalid_instruction(opcode),
            },
            9 => match opcode.get_nibble(3) {
                0 => self.skip_if_ne_reg(opcode.get_nibble(1), opcode.get_nibble(2)),
                _ => Chip8Emulator::invalid_instruction(opcode),
            },
            0xa => self.store_address(opcode.get_nibbles_from(1)),
            0xb => self.jump_to_plus_v0(opcode.get_nibbles_from(1)),
            0xc => self.store_random(opcode.get_nibble(1), opcode.get_nibbles_from(2) as u8),
            0xd => self.draw_sprite(
                opcode.get_nibble(1),
                opcode.get_nibble(2),
                opcode.get_nibble(3),
            ),
            0xe => match opcode.get_nibbles_from(2) {
                0x9e => self.skip_if_pressed(opcode.get_nibble(1)),
                0xa1 => self.skip_if_not_pressed(opcode.get_nibble(1)),
                _ => Chip8Emulator::invalid_instruction(opcode),
            },
            0xf => match opcode.get_nibbles_from(2) {
                0x07 => self.store_delay(opcode.get_nibble(1)),
                0x0a => self.wait_for_keypress(opcode.get_nibble(1)),
                _ => Chip8Emulator::invalid_instruction(opcode),
            },
            _ => Chip8Emulator::invalid_instruction(opcode),
        }
    }

    fn get_next_opcode(&mut self) -> Opcode {
        let opcode = ((self.memory[self.pc as usize] as u16) << 8)
            + self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;
        Opcode::new(opcode)
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

    fn skip_if_ne_reg(&mut self, x: u8, y: u8) {
        if self.V[x as usize] != self.V[y as usize] {
            self.pc += 2;
        }
    }

    fn store_address(&mut self, address: u16) {
        self.I = address;
    }

    fn jump_to_plus_v0(&mut self, address: u16) {
        self.jump_to(address + self.V[0] as u16)
    }

    fn store_random(&mut self, x: u8, mask: u8) {
        self.V[x as usize] = rand::random::<u8>() & mask;
    }

    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        let x = self.V[x as usize] % WIDTH;
        let y = self.V[y as usize] % HEIGHT;

        self.V[0xf] = 0;

        for dy in 0..n {
            let row = self.memory[self.I as usize + dy as usize];
            for dx in 0..8 {
                if (row >> (7 - dx) & 1) == 1 {
                    if self.gfx.toggle((x + dx) as u32, (y + dy) as u32) {
                        self.V[0xf] = 1;
                    }
                }
            }
        }
    }

    fn skip_if_pressed(&mut self, x: u8) {
        if self.keypad.is_key_down(self.V[x as usize]) {
            self.pc += 2;
        }
    }

    fn skip_if_not_pressed(&mut self, x: u8) {
        if !self.keypad.is_key_down(self.V[x as usize]) {
            self.pc += 2;
        }
    }

    fn store_delay(&mut self, x: u8) {
        self.V[x as usize] = self.delay_timer.value();
    }

    fn wait_for_keypress(&mut self, x: u8) {
        self.waiting_for_keypress = Some(x);
    }

    fn invalid_instruction(opcode: Opcode) {
        web_sys::console::error_1(&format!("Invalid instruction {:X}", opcode.value()).into())
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

        assert_eq!(chip8.get_gfx_width(), WIDTH as u32);
        assert_eq!(chip8.get_gfx_height(), HEIGHT as u32);
        assert!(chip8.gfx_needs_rerender());
        assert!(!chip8.gfx_needs_rerender());
        assert!(!chip8.get_gfx_pixel(5, 5));
        chip8.gfx.toggle(5, 5);
        assert!(chip8.get_gfx_pixel(5, 5));
        assert!(chip8.gfx.toggle(5, 5));
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
            assert_eq!(chip8.get_next_opcode().value(), *opcode)
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
        chip8.skip_if_ne_reg(0, 1);
        assert_eq!(chip8.pc, 6);
        chip8.skip_if_ne_reg(0, 2);
        assert_eq!(chip8.pc, 8);

        chip8.store_address(0xad13);
        assert_eq!(chip8.I, 0xad13);
        chip8.store(0, 5);
        chip8.jump_to_plus_v0(10);
        assert_eq!(chip8.pc, 15);

        chip8.keydown(0xa);
        chip8.skip_if_pressed(0xb);
        assert_eq!(chip8.pc, 15);
        chip8.skip_if_not_pressed(0xa);
        assert_eq!(chip8.pc, 15);
        chip8.skip_if_pressed(0xa);
        assert_eq!(chip8.pc, 17);
        chip8.skip_if_not_pressed(0xb);
        assert_eq!(chip8.pc, 19);
        chip8.keyup(0xa);
        chip8.skip_if_pressed(0xa);
        assert_eq!(chip8.pc, 19);
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
        chip8.add(x, Opcode::new(0x7005).get_nibbles_from(2) as u8);
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

    #[test]
    fn test_rand() {
        let mut chip8 = Chip8Emulator::new(0.0);
        for _ in 1..10 {
            chip8.store_random(0, 0xff);
            assert!(chip8.V[0] <= std::u8::MAX);
            assert!(chip8.V[0] >= std::u8::MIN);
        }
    }

    #[test]
    fn test_draw_sprite() {
        let mut chip8 = Chip8Emulator::new(0.0);
        assert!(chip8.gfx_needs_rerender());

        chip8.store(0, 10);
        chip8.store(1, 10);

        chip8.draw_sprite(0, 1, 3);
        assert!(!chip8.gfx_needs_rerender());
        assert_eq!(chip8.V[0xf], 0);

        chip8.memory[5] = 0b11110000;
        chip8.memory[6] = 0b00001111;
        chip8.memory[7] = 0b10101010;
        chip8.store_address(5);

        chip8.draw_sprite(0, 1, 3);
        assert!(chip8.gfx_needs_rerender());
        assert_eq!(chip8.V[0xf], 0);
        assert_eq!(chip8.get_gfx_pixel(10, 10), true);
        assert_eq!(chip8.get_gfx_pixel(11, 10), true);
        assert_eq!(chip8.get_gfx_pixel(14, 10), false);
        assert_eq!(chip8.get_gfx_pixel(14, 11), true);
        assert_eq!(chip8.get_gfx_pixel(13, 11), false);
        assert_eq!(chip8.get_gfx_pixel(10, 12), true);
        assert_eq!(chip8.get_gfx_pixel(11, 12), false);

        chip8.draw_sprite(0, 1, 3);
        assert!(chip8.gfx_needs_rerender());
        assert_eq!(chip8.V[0xf], 1);
        assert_eq!(chip8.get_gfx_pixel(10, 10), false);
        assert_eq!(chip8.get_gfx_pixel(11, 10), false);
        assert_eq!(chip8.get_gfx_pixel(14, 10), false);
        assert_eq!(chip8.get_gfx_pixel(14, 11), false);
        assert_eq!(chip8.get_gfx_pixel(13, 11), false);
        assert_eq!(chip8.get_gfx_pixel(10, 12), false);
        assert_eq!(chip8.get_gfx_pixel(11, 12), false);
    }
}

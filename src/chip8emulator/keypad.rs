pub struct KeyPad {
    state: [bool; 16],
}

impl KeyPad {
    pub fn new() -> KeyPad {
        KeyPad {
            state: [false; 16],
        }
    }

    pub fn keydown(&mut self, key: u8) {
        KeyPad::check_key_in_range(key);
        self.state[key as usize] = true;
    }

    pub fn keyup(&mut self, key: u8) {
        KeyPad::check_key_in_range(key);
        self.state[key as usize] = false;
    }

    pub fn is_key_down(&self, key: u8) -> bool {
        KeyPad::check_key_in_range(key);
        self.state[key as usize]
    }

    fn check_key_in_range(key: u8) {
        assert!(key <= 0xf, "{:X} is not a key on the keypad", key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypad() {
        let mut keypad = KeyPad::new();
        assert!(!keypad.is_key_down(0xa));

        keypad.keydown(0xa);
        assert!(keypad.is_key_down(0xa));

        keypad.keyup(0xa);
        assert!(!keypad.is_key_down(0xa));
    }
}
pub struct Opcode(u16);

impl Opcode {
    pub fn new(opcode: u16) -> Opcode {
        Opcode(opcode)
    }

    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn get_nibble(&self, index: u8) -> u8 {
        self.get_nibbles(index, index + 1) as u8
    }

    pub fn get_nibbles(&self, start_index: u8, end_index: u8) -> u16 {
        assert!(start_index < end_index);
        assert!(end_index <= 4);

        let mut mask = 0;
        for i in start_index as u16..end_index as u16 {
            mask += 0xf << (3 - i) * 4;
        }
        (self.0 & mask) >> (4 - end_index as u16) * 4
    }

    pub fn get_nibbles_from(&self, start_index: u8) -> u16 {
        self.get_nibbles(start_index, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_nibbles() {
        let opcode = Opcode::new(0x3a7b);
        assert_eq!(opcode.get_nibbles(1, 3), 0xa7);
        assert_eq!(opcode.get_nibbles(2, 3), 0x7);
        assert_eq!(opcode.get_nibbles(2, 4), 0x7b);
        assert_eq!(opcode.get_nibbles(0, 4), 0x3a7b);
        assert_eq!(opcode.get_nibble(0), 0x3);
        assert_eq!(opcode.get_nibble(3), 0xb);
    }
}

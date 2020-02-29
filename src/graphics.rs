use fixedbitset::FixedBitSet;

pub struct Graphics {
    width: usize,
    height: usize,
    display: FixedBitSet,
}

impl Graphics {
    pub fn new() -> Graphics {
        let width = 64;
        let height = 32;
        let display = FixedBitSet::with_capacity(width * height);
        Graphics { width, height, display }
    }

    /// Toggles the pixel at column `x` and row `y` (0-indexed) on the display
    /// and returns whether a pixel was toggled from on to off.
    pub fn toggle(&mut self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            panic!("Pixel ({}, {}) is out of bounds of display size {}x{}",
                   x, y, self.width, self.height);
        }
        let index = y * self.width + x;
        let res = self.display[index];
        self.display.toggle(index);
        res
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
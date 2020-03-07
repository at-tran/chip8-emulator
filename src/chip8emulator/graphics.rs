use fixedbitset::FixedBitSet;

pub struct Graphics {
    width: usize,
    height: usize,
    display: FixedBitSet,
    changed: bool,
}

impl Graphics {
    pub fn new(width: usize, height: usize) -> Graphics {
        let display = FixedBitSet::with_capacity(width * height);
        Graphics { width, height, display, changed: false }
    }

    /// Toggles the pixel at column `x` and row `y` (0-indexed) on the display
    /// and returns whether a pixel was toggled from on to off.
    pub fn toggle(&mut self, x: usize, y: usize) -> bool {
        assert!(x < self.width && y < self.height,
                "Pixel ({}, {}) is out of bounds of display size {}x{}",
                x, y, self.width, self.height);

        let index = y * self.width + x;
        let res = self.display[index];
        self.display.toggle(index);
        self.changed = true;
        res
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.display[y * self.width + x]
    }

    pub fn needs_rerender(&mut self) -> bool {
        let res = self.changed;
        self.changed = false;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_toggle() {
        let mut gfx = Graphics::new(2, 2);

        gfx.display.insert(2);
        assert_eq!(gfx.toggle(0, 1), true);
        assert_eq!(gfx.toggle(0, 1), false);
        assert!(gfx.display[2]);
        assert_eq!(gfx.toggle(0, 1), true);
        assert_eq!(gfx.toggle(1, 1), false);
        assert!(gfx.display[3]);
        assert_eq!(gfx.toggle(0, 0), false);
        assert_eq!(gfx.toggle(1, 1), true);
        assert!(gfx.get_pixel(0, 0) && !gfx.get_pixel(0, 1) &&
            !gfx.get_pixel(1, 0) && !gfx.get_pixel(1, 1));
    }
}
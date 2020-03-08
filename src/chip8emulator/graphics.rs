use fixedbitset::FixedBitSet;

pub struct Graphics {
    width: u32,
    height: u32,
    display: FixedBitSet,
    changed: bool,
}

impl Graphics {
    pub fn new(width: u32, height: u32) -> Graphics {
        let display = FixedBitSet::with_capacity((width * height) as usize);
        Graphics { width, height, display, changed: true }
    }

    /// Toggles the pixel at column `x` and row `y` (0-indexed) on the display
    /// and returns whether a pixel was toggled from on to off.
    pub fn toggle(&mut self, x: u32, y: u32) -> bool {
        assert!(x < self.width && y < self.height,
                "Pixel ({}, {}) is out of bounds of display size {}x{}",
                x, y, self.width, self.height);

        let index = y * self.width + x;
        let res = self.display[index as usize];
        self.display.toggle(index as usize);
        self.changed = true;
        res
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> bool {
        self.display[(y * self.width + x) as usize]
    }

    pub fn needs_rerender(&mut self) -> bool {
        let res = self.changed;
        self.changed = false;
        res
    }

    pub fn clear(&mut self) {
        self.display.clear();
        self.changed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_toggle() {
        let mut gfx = Graphics::new(2, 2);

        assert_eq!(gfx.get_width(), 2);
        assert_eq!(gfx.get_height(), 2);

        gfx.display.insert(2);
        assert_eq!(gfx.toggle(0, 1), true);
        assert!(gfx.needs_rerender());
        assert!(!gfx.needs_rerender());
        assert_eq!(gfx.toggle(0, 1), false);
        assert!(gfx.display[2]);
        assert!(gfx.needs_rerender());
        assert_eq!(gfx.toggle(0, 1), true);
        assert_eq!(gfx.toggle(1, 1), false);
        assert!(gfx.display[3]);
        assert_eq!(gfx.toggle(0, 0), false);
        assert_eq!(gfx.toggle(1, 1), true);
        assert!(gfx.get_pixel(0, 0) && !gfx.get_pixel(0, 1) &&
            !gfx.get_pixel(1, 0) && !gfx.get_pixel(1, 1));

        gfx.clear();
        assert!(!gfx.get_pixel(0, 0) && !gfx.get_pixel(0, 1) &&
            !gfx.get_pixel(1, 0) && !gfx.get_pixel(1, 1));

    }
}
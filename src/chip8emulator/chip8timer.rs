use super::timer::Timer;

pub struct Chip8Timer {
    value: u8,
    timer: Timer,
}

impl Chip8Timer {
    pub fn new(current_time: f64) -> Chip8Timer {
        Chip8Timer {
            value: 0,
            timer: Timer::new(current_time, 1000.0 / 60.0),
        }
    }

    pub fn step(&mut self, current_time: f64) {
        let ticks = self.timer.step(current_time).min(std::u8::MAX as u32);
        self.value = self.value.saturating_sub(ticks as u8);
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn set_value(&mut self, value: u8) {
        self.value = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let t = 10.0;
        let interval = 1000.0 / 60.0;
        let mut timer = Chip8Timer::new(t);
        assert_eq!(timer.value(), 0);
        timer.set_value(5);

        timer.step(t + 0.99 * interval);
        assert_eq!(timer.value(), 5);
        timer.step(t + 1.1 * interval);
        assert_eq!(timer.value(), 4);

        timer.step(t + 3.01 * interval);
        assert_eq!(timer.value(), 2);

        timer.step(t + 5.01 * interval);
        assert_eq!(timer.value(), 0);
    }
}

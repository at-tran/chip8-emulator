pub struct Timer {
    value: u8,
    prev_time: f64,
}

impl Timer {
    const INTERVAL: f64 = 1000.0 / 60.0;

    pub fn new(current_time: f64) -> Timer {
        Timer {
            value: 0,
            prev_time: current_time,
        }
    }

    pub fn step(&mut self, current_time: f64) {
        let ticks = (current_time - self.prev_time) / Timer::INTERVAL;
        assert!(ticks >= 0.0, "Current time less than previous time");
        self.value = self.value.saturating_sub(ticks as u8);
        self.prev_time += ticks.floor() * Timer::INTERVAL;
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
        let mut timer = Timer::new(t);
        assert_eq!(timer.value(), 0);
        timer.set_value(5);

        timer.step(t + 0.99 * Timer::INTERVAL);
        assert_eq!(timer.value(), 5);
        timer.step(t + 1.1 * Timer::INTERVAL);
        assert_eq!(timer.value(), 4);

        timer.step(t + 3.01 * Timer::INTERVAL);
        assert_eq!(timer.value(), 2);

        timer.step(t + 5.01 * Timer::INTERVAL);
        assert_eq!(timer.value(), 0);
    }
}
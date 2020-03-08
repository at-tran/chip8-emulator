pub struct Timer {
    prev_time: f64,
    interval: f64,
}

impl Timer {
    pub fn new(current_time: f64, interval: f64) -> Timer {
        Timer {
            prev_time: current_time,
            interval,
        }
    }

    pub fn step(&mut self, current_time: f64) -> u32 {
        let ticks = (current_time - self.prev_time) / self.interval;
        assert!(ticks >= 0.0, "Current time less than previous time");
        self.prev_time += ticks.floor() * self.interval;
        ticks as u32
    }

    pub fn set_interval(&mut self, interval: f64) {
        self.interval = interval;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let t = 10.0;
        let interval = 5.0;
        let mut timer = Timer::new(t, interval);
        assert_eq!(timer.step(t), 0);

        assert_eq!(timer.step(t + 0.99 * interval), 0);
        assert_eq!(timer.step(t + 1.1 * interval), 1);

        assert_eq!(timer.step(t + 3.01 * interval), 2);

        let t = t + 5.01 * interval;
        assert_eq!(timer.step(t), 2);

        let interval = 15.0;
        timer.set_interval(interval);
        assert_eq!(timer.step(t + 0.99 * interval), 0);
        assert_eq!(timer.step(t + 2.99 * interval), 2);
    }
}
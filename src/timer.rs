use wasm_bindgen::prelude::*;

pub struct Timer {
    value: u8,
    prev_time: f64,
}

impl Timer {
    const INTERVAL: f64 = 1000.0 / 60.0;

    pub fn new() -> Timer {
        Timer {
            value: 0,
            prev_time: now(),
        }
    }

    pub fn step(&mut self) {
        let ticks = (now() - self.prev_time) / Timer::INTERVAL;
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

thread_local! {
    static performance: web_sys::Performance =
        web_sys::window().unwrap().performance().unwrap();
}

fn now() -> f64 {
    performance.with(|p| { p.now() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_timer() {
        let mut timer = Timer::new();
        assert_eq!(timer.value(), 0);
        timer.set_value(5);

        let t = now();
        // Add 2ms to give enough for timer.step()
        while now() + 2.0 - t < Timer::INTERVAL { timer.step(); }
        assert_eq!(timer.value(), 5);
        while now() + 2.0 - t < 2.0 * Timer::INTERVAL { timer.step(); }
        assert_eq!(timer.value(), 4);

        let t = now();
        while now() - t < 2.0 * Timer::INTERVAL {}
        timer.step();
        assert_eq!(timer.value(), 2);

        while now() - t < 5.0 * Timer::INTERVAL { timer.step(); }
        assert_eq!(timer.value(), 0);
    }
}
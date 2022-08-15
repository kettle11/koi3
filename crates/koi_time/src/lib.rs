/// Time elapsed since last draw call
pub struct Time {
    pub fixed_time_step_seconds: f64,
    time_accumulator_seconds: f64,
    last_time_step: kinstant::Instant,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    pub fn new() -> Self {
        Self {
            fixed_time_step_seconds: 1.0 / 60.0,
            time_accumulator_seconds: 0.0,
            last_time_step: kinstant::Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let now = kinstant::Instant::now();
        let elapsed = now - self.last_time_step;
        self.time_accumulator_seconds += elapsed.as_secs_f64();
        self.last_time_step = now;
    }

    pub fn fixed_update_ready(&mut self) -> bool {
        if self.time_accumulator_seconds >= self.fixed_time_step_seconds {
            self.time_accumulator_seconds -= self.fixed_time_step_seconds;
            true
        } else {
            false
        }
    }
}

/// Time elapsed since last draw call
pub struct Time {
    pub fixed_time_step_seconds: f64,
    /// Seconds between the last draw and this draw.
    pub draw_delta_seconds: f64,
    time_accumulator_seconds: f64,
    last_time_step: kinstant::Instant,
    last_draw_time_stamp: kinstant::Instant,
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
            draw_delta_seconds: 0.0,
            last_time_step: kinstant::Instant::now(),
            last_draw_time_stamp: kinstant::Instant::now(),
        }
    }

    pub fn reset_accumulator(&mut self) {
        self.last_time_step = kinstant::Instant::now();
        self.time_accumulator_seconds = 0.0;
    }

    pub fn update(&mut self) {
        let now = kinstant::Instant::now();
        let elapsed = now - self.last_time_step;
        self.time_accumulator_seconds += elapsed.as_secs_f64();
        self.last_time_step = now;
    }

    pub fn update_draw(&mut self) {
        let now = kinstant::Instant::now();
        let elapsed = now - self.last_draw_time_stamp;
        self.draw_delta_seconds = elapsed.as_secs_f64();
        self.last_draw_time_stamp = now;
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

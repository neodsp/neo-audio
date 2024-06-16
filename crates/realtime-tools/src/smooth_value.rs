pub use easer::functions::*;

pub struct SmoothValue {
    num_steps: usize,
    counter: usize,
    begin: f32,
    change: f32,
    current: f32,
    target: f32,
    easer: fn(f32, f32, f32, f32) -> f32,
}

impl SmoothValue {
    pub fn new(initial_value: f32, easer: fn(f32, f32, f32, f32) -> f32) -> Self {
        Self {
            num_steps: 0,
            counter: 0,
            begin: initial_value,
            change: 0.0,
            current: initial_value,
            target: initial_value,
            easer,
        }
    }

    pub fn prepare(&mut self, sample_rate: u32, ramp_time_ms: usize) {
        self.num_steps = ((sample_rate as f64 / 1000.0) * ramp_time_ms as f64).round() as usize;
    }

    pub fn set_num_steps(&mut self, num_steps: usize) {
        self.num_steps = num_steps;
    }

    pub fn set_target_value(&mut self, target: f32) {
        self.target = target;
        self.begin = self.current;
        self.change = target - self.current;
        self.counter = 0;
    }

    pub fn set_current_and_target_value(&mut self, target: f32) {
        self.target = target;
        self.current = target;
        self.begin = target;
        self.change = 0.0;
        self.counter = self.num_steps + 1;
    }

    pub fn next_value(&mut self) -> f32 {
        if self.counter > self.num_steps {
            self.target
        } else {
            self.current = (self.easer)(
                self.counter as f32,
                self.begin,
                self.change,
                self.num_steps as f32,
            );
            self.counter += 1;
            self.current
        }
    }

    pub fn skip(&mut self, steps: usize) -> f32 {
        self.counter += steps;
        if self.counter > self.num_steps {
            self.target
        } else {
            self.current = (self.easer)(
                self.counter as f32,
                self.begin,
                self.change,
                self.num_steps as f32,
            );
            self.current
        }
    }
}

#[test]
fn test() {
    let mut smooth_value = SmoothValue::new(0.0, Linear::ease_in_out);
    smooth_value.prepare(100, 100);
    smooth_value.set_target_value(1.0);
    for i in 0..=10 {
        assert_eq!(smooth_value.next_value(), i as f32 / 10.0);
    }
}

use std::collections::VecDeque;

const MINUS_INF_DB: f32 = -100.0;

pub struct Level {
    pub peak_db: f32,
    pub rms_db: f32,
}

pub struct LevelMeter {
    num_samples: usize,
    store: VecDeque<f32>,
    send_fn: Box<dyn Fn(Level) + Send>,
}

impl LevelMeter {
    pub fn new(send_fn: impl Fn(Level) + Send + 'static) -> Self {
        Self {
            num_samples: 0,
            store: VecDeque::new(),
            send_fn: Box::new(send_fn),
        }
    }

    pub fn prepare(&mut self, sample_rate: u32, num_frames: u32, ms: u32) {
        self.num_samples = (sample_rate / 1000 * ms) as usize;
        self.store.reserve(self.num_samples + num_frames as usize);
    }

    pub fn process<'a>(&mut self, audio_data: impl IntoIterator<Item = &'a f32>) {
        for sample in audio_data.into_iter() {
            self.store.push_back(*sample);
        }

        if self.store.len() >= self.num_samples {
            let mut peak = 0.0;
            let mut rms = 0.0;
            for _ in 0..self.num_samples {
                let sample = self.store.pop_front().unwrap().abs();
                if sample > peak {
                    peak = sample;
                }
                rms += sample * sample;
            }

            let peak_db = if peak == 0.0 {
                MINUS_INF_DB
            } else {
                20. * peak.log10()
            };

            rms /= self.num_samples as f32;
            let rms_db = if rms == 0.0 {
                MINUS_INF_DB
            } else {
                20. * rms.log10()
            };

            (self.send_fn)(Level { peak_db, rms_db });
        }
    }
}

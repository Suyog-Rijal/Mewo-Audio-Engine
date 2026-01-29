pub struct Limiter {
    threshold: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
    gain: f32,
    smoothing_coeff: f32,
}

impl Limiter {
    pub fn new(threshold_db: f32, sample_rate: f32) -> Self {
        let threshold = 10.0f32.powf(threshold_db / 20.0);
        let attack_time = 0.01;
        let release_time = 0.25;
        let smoothing_time = 0.01;

        Self {
            threshold,
            attack_coeff: (-1.0 / (sample_rate * attack_time)).exp(),
            release_coeff: (-1.0 / (sample_rate * release_time)).exp(),
            smoothing_coeff: (-1.0 / (sample_rate * smoothing_time)).exp(),
            envelope: 0.0,
            gain: 1.0,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let x = input.abs() + 1e-10;

        if x > self.envelope {
            self.envelope = self.attack_coeff * (self.envelope - x) + x;
        } else {
            self.envelope = self.release_coeff * (self.envelope - x) + x;
        }

        let target_gain = if self.envelope > self.threshold {
            self.threshold / self.envelope
        } else {
            1.0
        };

        self.gain = self.smoothing_coeff * (self.gain - target_gain) + target_gain;

        input * self.gain
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain = 1.0;
    }
}
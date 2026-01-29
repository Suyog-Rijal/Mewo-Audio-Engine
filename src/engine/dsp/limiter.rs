pub struct Limiter {
    threshold: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
}

impl Limiter {
    pub fn new(threshold_db: f32, sample_rate: f32) -> Self {
        let threshold = 10.0f32.powf(threshold_db / 20.0);
        let attack_time = 0.005;
        let release_time = 0.15;

        Self {
            threshold,
            attack_coeff: (-1.0 / (sample_rate * attack_time)).exp(),
            release_coeff: (-1.0 / (sample_rate * release_time)).exp(),
            envelope: 0.0,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let x = input.abs();

        if x > self.envelope {
            self.envelope = self.attack_coeff * (self.envelope - x) + x;
        } else {
            self.envelope = self.release_coeff * (self.envelope - x) + x;
        }

        let gain = if self.envelope > self.threshold {
            self.threshold / self.envelope
        } else {
            1.0
        };

        input * gain
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

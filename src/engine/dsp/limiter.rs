pub struct Limiter {
    threshold: f32,
    attack: f32,
    release: f32,
    envelope: f32,
}

impl Limiter {
    pub fn new(threshold_db: f32, sample_rate: f32) -> Self {
        // Threshold: 0.0 dB is 1.0 amplitude
        let threshold = 10.0f32.powf(threshold_db / 20.0);
        
        // Attack/Release times in seconds (very fast for limiter)
        let attack_time = 0.001; // 1ms
        let release_time = 0.1; // 100ms
        
        Self {
            threshold,
            attack: (-1.0 / (sample_rate * attack_time)).exp(),
            release: (-1.0 / (sample_rate * release_time)).exp(),
            envelope: 0.0,
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let input_abs = sample.abs();
        
        // Envelope follower
        if input_abs > self.envelope {
            self.envelope = self.attack * (self.envelope - input_abs) + input_abs;
        } else {
            self.envelope = self.release * (self.envelope - input_abs) + input_abs;
        }
        
        // Gain reduction
        let gain = if self.envelope > self.threshold {
            self.threshold / self.envelope
        } else {
            1.0
        };
        
        sample * gain
    }
}

use crate::engine::dsp::biquad::{BiquadFilter, FilterType};

pub struct BassProcessor {
    high_pass: Vec<BiquadFilter>,
    shelf: Vec<BiquadFilter>,
    channels: usize,
    sample_rate: f32,
    low_energy: Vec<f32>,
    total_energy: Vec<f32>,
    count: usize,
    target_gain: f32,
    current_gain: f32,
    enabled: bool,
    intensity: f32,
}

impl BassProcessor {
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        let mut high_pass = Vec::new();
        let mut shelf = Vec::new();

        for _ in 0..channels {
            high_pass.push(BiquadFilter::new(FilterType::HighPass, sample_rate, 30.0, 0.707, 0.0));
            shelf.push(BiquadFilter::new(FilterType::LowShelf, sample_rate, 60.0, 0.6, 0.0));
        }

        Self {
            high_pass,
            shelf,
            channels,
            sample_rate,
            low_energy: vec![0.0; channels],
            total_energy: vec![0.0; channels],
            count: 0,
            target_gain: 0.0,
            current_gain: 0.0,
            enabled: false,
            intensity: 50.0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 100.0);
    }

    fn update_gain(&mut self) {
        let diff = self.target_gain - self.current_gain;
        if diff.abs() > 0.0001 {
            self.current_gain += diff * 0.005;
            for ch in 0..self.channels {
                self.shelf[ch].update(
                    FilterType::LowShelf,
                    self.sample_rate,
                    60.0,
                    0.6,
                    self.current_gain,
                );
            }
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        let frames = samples.len() / self.channels;
        self.update_gain();

        for i in 0..frames {
            for ch in 0..self.channels {
                let idx = i * self.channels + ch;
                let input = samples[idx];

                self.total_energy[ch] += input * input;
                self.low_energy[ch] += input * input;

                let mut x = self.high_pass[ch].process(input);
                x = self.shelf[ch].process(x);

                samples[idx] = x;
            }
            self.count += 1;
        }

        if self.count >= 2048 {
            self.adapt();
        }
    }

    fn adapt(&mut self) {
        if !self.enabled {
            self.target_gain = 0.0;
            self.count = 0;
            return;
        }

        let max_gain = (self.intensity / 100.0) * 8.0;

        let mut bass_ratio = 0.0;
        let mut total = 0.0;

        for ch in 0..self.channels {
            let t = self.total_energy[ch] / self.count as f32;
            let l = self.low_energy[ch] / self.count as f32;

            if t > 0.000001 {
                bass_ratio += (l / t).sqrt();
            }

            total += t;
            self.total_energy[ch] = 0.0;
            self.low_energy[ch] = 0.0;
        }

        bass_ratio /= self.channels as f32;
        total /= self.channels as f32;

        if total > 0.0001 {
            if bass_ratio < 0.4 {
                self.target_gain = (self.target_gain + 0.2).min(max_gain);
            } else if bass_ratio > 0.6 {
                self.target_gain = (self.target_gain - 0.2).max(0.0);
            }
        }

        self.count = 0;
    }
}
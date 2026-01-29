use crate::engine::dsp::biquad::{BiquadFilter, FilterType};

pub struct HighFreqEQ {
    filters: Vec<BiquadFilter>,
    channels: usize,
}

impl HighFreqEQ {
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        let mut filters = Vec::with_capacity(channels);
        for _ in 0..channels {
            filters.push(BiquadFilter::new(
                FilterType::HighShelf,
                sample_rate,
                12000.0,
                0.7,
                -1.5,
            ));
        }

        Self { filters, channels }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        let frames = samples.len() / self.channels;

        for i in 0..frames {
            for ch in 0..self.channels {
                let idx = i * self.channels + ch;
                samples[idx] = self.filters[ch].process(samples[idx]);
            }
        }
    }
}
use crate::engine::dsp::bass::BassProcessor;
use crate::engine::dsp::eq::HighFreqEQ;
use crate::engine::dsp::limiter::Limiter;

pub struct DspChain {
    pub(crate) bass: BassProcessor,
    hf_eq: HighFreqEQ,
    limiter: Vec<Limiter>,
    channels: usize,
}

impl DspChain {
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        let mut limiter = Vec::new();
        for _ in 0..channels {
            limiter.push(Limiter::new(-0.1, sample_rate));
        }

        Self {
            bass: BassProcessor::new(sample_rate, channels),
            hf_eq: HighFreqEQ::new(sample_rate, channels),
            limiter,
            channels,
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        self.bass.process(samples);
        self.hf_eq.process(samples);

        let frames = samples.len() / self.channels;
        for i in 0..frames {
            for ch in 0..self.channels {
                let idx = i * self.channels + ch;
                samples[idx] = self.limiter[ch].process(samples[idx]);
            }
        }
    }
}
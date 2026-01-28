use crate::engine::dsp::biquad::{BiquadFilter, FilterType};
use crate::engine::dsp::limiter::Limiter;

pub struct BassProcessor {
    high_passes: Vec<BiquadFilter>,
    low_shelves: Vec<BiquadFilter>,
    limiters: Vec<Limiter>,
    channels: usize,
    sample_rate: f32,
    
    // Adaptive bass state
    low_energy_accumulator: Vec<f32>,
    total_energy_accumulator: Vec<f32>,
    sample_count: usize,
    target_bass_gain: f32,
    current_bass_gain: f32,
    enabled: bool,
    intensity: f32, // 0.0 to 100.0
}

impl BassProcessor {
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        let mut high_passes = Vec::with_capacity(channels);
        let mut low_shelves = Vec::with_capacity(channels);
        let mut limiters = Vec::with_capacity(channels);

        for _ in 0..channels {
            // High-pass: 30Hz, Q=0.707 (Butterworth-ish)
            high_passes.push(BiquadFilter::new(FilterType::HighPass, sample_rate, 30.0, 0.707, 0.0));
            // Low-shelf: 100Hz, Q=0.7, 0dB gain initially
            low_shelves.push(BiquadFilter::new(FilterType::LowShelf, sample_rate, 100.0, 0.7, 0.0));
            // Limiter: -0.1dB threshold
            limiters.push(Limiter::new(-0.1, sample_rate));
        }

        Self {
            high_passes,
            low_shelves,
            limiters,
            channels,
            sample_rate,
            low_energy_accumulator: vec![0.0; channels],
            total_energy_accumulator: vec![0.0; channels],
            sample_count: 0,
            target_bass_gain: 0.0,
            current_bass_gain: 0.0,
            enabled: false,
            intensity: 50.0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.target_bass_gain = 0.0;
        }
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 100.0);
    }

    fn update_filters(&mut self) {
        // Smoothly interpolate current gain towards target gain
        // This prevents pops and adds a natural feel to adaptive changes
        let smoothing_factor = 0.001; // Very slow smoothing per sample
        
        let diff = self.target_bass_gain - self.current_bass_gain;
        if diff.abs() > 0.001 {
            self.current_bass_gain += diff * smoothing_factor;
            
            for ch in 0..self.channels {
                self.low_shelves[ch].update_coefficients(
                    FilterType::LowShelf,
                    self.sample_rate,
                    100.0,
                    0.7,
                    self.current_bass_gain
                );
            }
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        let frames = samples.len() / self.channels;
        
        // Update filters for smoothing
        self.update_filters();

        for i in 0..frames {
            for ch in 0..self.channels {
                let idx = i * self.channels + ch;
                let input_sample = samples[idx];
                let mut sample = input_sample;

                // 1. Measure total energy (pre-HPF)
                self.total_energy_accumulator[ch] += input_sample.abs();
                
                // 2. High-pass (remove rumble)
                sample = self.high_passes[ch].process(sample);
                
                // 3. Measure low energy (post-HPF, but before boost)
                self.low_energy_accumulator[ch] += sample.abs();

                // 4. Low-shelf (bass boost)
                sample = self.low_shelves[ch].process(sample);

                // 5. Limiter (protect output)
                sample = self.limiters[ch].process(sample);

                samples[idx] = sample;
            }
            self.sample_count += 1;
        }

        // Adaptive logic: Every 2048 frames (approx 46ms at 44.1kHz), adjust target gain
        if self.sample_count >= 2048 {
            self.update_adaptive_gain();
        }
    }

    fn update_adaptive_gain(&mut self) {
        if !self.enabled {
            self.target_bass_gain = 0.0;
            self.sample_count = 0;
            return;
        }

        let max_possible_gain = (self.intensity / 100.0) * 8.0; // Max 8dB boost at 100% intensity

        let mut avg_bass_ratio = 0.0;
        let mut total_avg_low = 0.0;
        let mut total_avg_total = 0.0;

        for ch in 0..self.channels {
            let avg_total = self.total_energy_accumulator[ch] / self.sample_count as f32;
            let avg_low = self.low_energy_accumulator[ch] / self.sample_count as f32;
            
            total_avg_total += avg_total;
            total_avg_low += avg_low;

            // Ratio of low energy to total energy
            let bass_ratio = if avg_total > 0.0001 { avg_low / avg_total } else { 0.0 };
            avg_bass_ratio += bass_ratio;

            // Reset accumulators
            self.total_energy_accumulator[ch] = 0.0;
            self.low_energy_accumulator[ch] = 0.0;
        }
        avg_bass_ratio /= self.channels as f32;
        total_avg_total /= self.channels as f32;

        // Adaptive logic:
        // Ideal bass ratio is around 0.4 - 0.5.
        // If below 0.35, we boost. If above 0.55, we cut back.
        // But only if there is significant signal (total > 0.001)
        if total_avg_total > 0.001 {
            if avg_bass_ratio < 0.35 {
                // Boost needed
                self.target_bass_gain = (self.target_bass_gain + 0.5).min(max_possible_gain);
            } else if avg_bass_ratio > 0.55 {
                // Already plenty of bass, or too much
                self.target_bass_gain = (self.target_bass_gain - 0.5).max(0.0);
            }
        } else {
            // Signal too weak to make a good judgment, hold current gain or drift to neutral
            // self.target_bass_gain = self.target_bass_gain * 0.95; 
        }
        
        // Ensure we don't exceed max possible gain if intensity changed
        if self.target_bass_gain > max_possible_gain {
            self.target_bass_gain = max_possible_gain;
        }

        self.sample_count = 0;
    }
}

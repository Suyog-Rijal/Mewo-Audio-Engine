use rubato::{Resampler as RubatoResampler, Fft, FixedSync};
use audioadapter_buffers::direct::SequentialSliceOfVecs;

pub struct Resampler {
    resampler: Fft<f32>,
    channels: usize,
    chunk_size: usize,
    buffer: Vec<f32>,
}

impl Resampler {
    pub fn new(
        source_sample_rate: u32,
        target_sample_rate: u32,
        channels: usize,
        chunk_size: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let resampler = Fft::<f32>::new(
            source_sample_rate as usize,
            target_sample_rate as usize,
            chunk_size,
            2,
            channels,
            FixedSync::Input,
        )?;

        Ok(Self {
            resampler,
            channels,
            chunk_size,
            buffer: Vec::with_capacity(chunk_size * channels),
        })
    }

    pub fn process(&mut self, input: &[f32]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        self.buffer.extend_from_slice(input);

        let mut all_output = Vec::new();

        while self.buffer.len() >= self.chunk_size * self.channels {
            let chunk: Vec<f32> = self.buffer.drain(0..self.chunk_size * self.channels).collect();
            let num_frames = self.chunk_size;

            let mut input_buffer = vec![vec![0.0; num_frames]; self.channels];
            for i in 0..num_frames {
                for ch in 0..self.channels {
                    input_buffer[ch][i] = chunk[i * self.channels + ch];
                }
            }

            let out_len = self.resampler.output_frames_next();
            let mut output_buffer = vec![vec![0.0; out_len]; self.channels];

            let input_adapter = SequentialSliceOfVecs::new(&input_buffer, self.channels, num_frames)?;
            let mut output_adapter = SequentialSliceOfVecs::new_mut(&mut output_buffer, self.channels, out_len)?;

            self.resampler.process_into_buffer(
                &input_adapter,
                &mut output_adapter,
                None,
            )?;

            for i in 0..out_len {
                for ch in 0..self.channels {
                    all_output.push(output_buffer[ch][i]);
                }
            }
        }

        Ok(all_output)
    }

    pub fn flush(&mut self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }

        let remaining_frames = self.buffer.len() / self.channels;
        let padding_needed = (self.chunk_size - remaining_frames) * self.channels;
        self.buffer.extend(vec![0.0; padding_needed]);

        self.process(&[])
    }

    pub fn input_frames_next(&self) -> usize {
        self.resampler.input_frames_next()
    }
}
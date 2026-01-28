use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig, SampleFormat, FromSample, Sample};
use std::sync::Arc;
use crate::engine::buffer::AudioBufferConsumer;
use crate::engine::clock::{Clock, PlaybackState};
use crate::engine::output::AudioOutput;

pub struct CpalBackend {
    _stream: Stream,
}

impl CpalBackend {
    pub fn new(
        mut consumer: AudioBufferConsumer,
        clock: Arc<Clock>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("No output device available")?;

        let config: StreamConfig = device.default_output_config()?.into();
        let sample_format = device.default_output_config()?.sample_format();

        // Update clock's sample rate and channels based on hardware
        clock.set_sample_rate(config.sample_rate);
        clock.set_channels(config.channels as u32);

        let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    process_audio(data, &mut consumer, &clock);
                },
                err_fn,
                None,
            )?,
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _| {
                    process_audio(data, &mut consumer, &clock);
                },
                err_fn,
                None,
            )?,
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _| {
                    process_audio(data, &mut consumer, &clock);
                },
                err_fn,
                None,
            )?,
            _ => return Err("Unsupported sample format".into()),
        };

        Ok(Self { _stream: stream })
    }
}

impl AudioOutput for CpalBackend {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self._stream.play()?;
        Ok(())
    }

    fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self._stream.pause()?;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self._stream.pause()?;
        // Additional cleanup if needed
        Ok(())
    }
}

/// Real-time safe audio processing callback.
fn process_audio<T: Sample + FromSample<f32>>(
    data: &mut [T],
    consumer: &mut AudioBufferConsumer,
    clock: &Arc<Clock>,
) {
    if clock.should_clear_buffer() {
        consumer.clear();
        clock.reset_clear_buffer();
    }

    if clock.get_state() != PlaybackState::Playing {
        for sample in data.iter_mut() {
            *sample = T::from_sample(0.0);
        }
        return;
    }

    let samples_read = consumer.pop_slice_f32(data);

    // Fill remaining with silence if underrun
    if samples_read < data.len() {
        for sample in &mut data[samples_read..] {
            *sample = T::from_sample(0.0);
        }
    }

    // Increment clock. We assume interleaved data, so we divide by channel count
    // to get sample position, but usually clock tracks "frames" or "total samples".
    // According to our Clock definition, it's sample_pos.
    clock.increment_samples(samples_read as u64);
}

/// Helper trait extension for AudioBufferConsumer to support generic sample conversion
/// since pop_slice works on f32 but we need to convert to T.
trait ConsumerExt {
    fn pop_slice_f32<T: Sample + FromSample<f32>>(&mut self, data: &mut [T]) -> usize;
}

impl ConsumerExt for AudioBufferConsumer {
    fn pop_slice_f32<T: Sample + FromSample<f32>>(&mut self, data: &mut [T]) -> usize {
        let mut count = 0;
        for out in data.iter_mut() {
            if let Some(sample) = self.pop() {
                *out = T::from_sample(sample);
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}

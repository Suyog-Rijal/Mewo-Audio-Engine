use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig, SampleFormat, FromSample, Sample};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::engine::buffer::AudioBufferConsumer;
use crate::engine::clock::{Clock, PlaybackState};
use crate::engine::output::AudioOutput;

pub struct CpalBackend {
    _stream: Stream,
    device_id: String,
    is_healthy: Arc<AtomicBool>,
    consumer: Arc<Mutex<Option<AudioBufferConsumer>>>,
}

impl CpalBackend {
    pub fn new(
        consumer: AudioBufferConsumer,
        clock: Arc<Clock>,
    ) -> Result<Self, (AudioBufferConsumer, Box<dyn std::error::Error>)> {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => return Err((consumer, "No output device available".into())),
        };

        let device_id = device.name().unwrap_or_else(|_| "unknown".to_string());
        let config_res = device.default_output_config();
        let config_inner = match config_res {
            Ok(c) => c,
            Err(e) => return Err((consumer, e.into())),
        };

        let sample_format = config_inner.sample_format();
        let config: StreamConfig = config_inner.into();

        clock.set_sample_rate(config.sample_rate);
        clock.set_channels(config.channels as u32);

        let is_healthy = Arc::new(AtomicBool::new(true));
        let is_healthy_err = is_healthy.clone();

        let err_fn = move |err| {
            is_healthy_err.store(false, Ordering::SeqCst);
        };

        let shared_consumer = Arc::new(Mutex::new(Some(consumer)));
        let consumer_for_callback = shared_consumer.clone();
        let clock_for_callback = clock.clone();

        let stream_res = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    if let Ok(mut guard) = consumer_for_callback.lock() {
                        if let Some(c) = guard.as_mut() {
                            process_audio(data, c, &clock_for_callback);
                        }
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _| {
                    if let Ok(mut guard) = consumer_for_callback.lock() {
                        if let Some(c) = guard.as_mut() {
                            process_audio(data, c, &clock_for_callback);
                        }
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _| {
                    if let Ok(mut guard) = consumer_for_callback.lock() {
                        if let Some(c) = guard.as_mut() {
                            process_audio(data, c, &clock_for_callback);
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => {
                let consumer = shared_consumer.lock().unwrap().take().unwrap();
                return Err((consumer, "Unsupported sample format".into()));
            }
        };

        match stream_res {
            Ok(stream) => Ok(Self {
                _stream: stream,
                device_id,
                is_healthy,
                consumer: shared_consumer,
            }),
            Err(e) => {
                let consumer = shared_consumer.lock().unwrap().take().unwrap();
                Err((consumer, e.into()))
            }
        }
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
        let _ = self._stream.pause();
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        if !self.is_healthy.load(Ordering::SeqCst) {
            return false;
        }
        let host = cpal::default_host();
        if let Some(device) = host.default_output_device() {
            if let Ok(name) = device.name() {
                if name != self.device_id {
                    return false;
                }
            }
        }
        true
    }

    fn shutdown(&mut self) -> Option<AudioBufferConsumer> {
        let _ = self._stream.pause();
        self.consumer.lock().ok()?.take()
    }

    fn tick(&mut self) {}
}

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

    if samples_read < data.len() {
        for sample in &mut data[samples_read..] {
            *sample = T::from_sample(0.0);
        }
    }

    clock.increment_samples(samples_read as u64);

    if samples_read == 0 && clock.is_eos() {
        clock.set_state(PlaybackState::Stopped);
    }
}

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
use std::path::Path;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};

use crate::engine::clock::{Clock, PlaybackState};
use crate::engine::decoder::{AudioDecoder, symphonia_decoder::SymphoniaDecoder};
use crate::engine::buffer::{create_audio_buffer, AudioBufferProducer};
use crate::engine::output::{AudioOutput, output_manager::OutputManager};
use crate::engine::dsp::resampler::Resampler;
use crate::engine::dsp::dsp_chain::DspChain;

enum DecoderCommand {
    Seek(f64),
    Stop,
    SetBassBoost(bool),
    SetBassIntensity(f32),
}

pub struct AudioEngine {
    clock: Arc<Clock>,
    output: Box<dyn AudioOutput + Send>,
    producer: Option<AudioBufferProducer>,
    decode_thread: Option<JoinHandle<()>>,
    is_decoding: Arc<AtomicBool>,
    command_tx: Option<Sender<DecoderCommand>>,
    bass_boost_enabled: Arc<AtomicBool>,
    bass_boost_intensity: Arc<std::sync::Mutex<f32>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let clock = Arc::new(Clock::new(44100));
        let (producer, consumer) = create_audio_buffer(44100 * 2);
        let output = Box::new(OutputManager::new(consumer, clock.clone()));

        Ok(Self {
            clock,
            output,
            producer: Some(producer),
            decode_thread: None,
            is_decoding: Arc::new(AtomicBool::new(false)),
            command_tx: None,
            bass_boost_enabled: Arc::new(AtomicBool::new(false)),
            bass_boost_intensity: Arc::new(std::sync::Mutex::new(50.0)),
        })
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();

        let mut decoder = SymphoniaDecoder::new(path)?;
        let mut producer = self.producer.take().ok_or("Producer missing")?;
        let is_decoding = self.is_decoding.clone();
        let clock = self.clock.clone();
        let bass_boost_enabled = self.bass_boost_enabled.clone();
        let bass_boost_intensity = self.bass_boost_intensity.clone();

        let mut output_rate = clock.get_sample_rate();
        let mut output_channels = clock.get_channels();

        let decoder_rate = decoder.sample_rate();
        let decoder_channels = decoder.channels() as usize;

        let mut resampler = if decoder_rate != output_rate || decoder_channels != output_channels as usize {
            Some(Resampler::new(decoder_rate, output_rate, decoder_channels, 1024)?)
        } else {
            None
        };

        let mut dsp = DspChain::new(output_rate as f32, output_channels as usize);
        dsp.bass.set_enabled(bass_boost_enabled.load(Ordering::SeqCst));
        if let Ok(v) = bass_boost_intensity.lock() {
            dsp.bass.set_intensity(*v);
        }

        let (tx, rx) = mpsc::channel();
        self.command_tx = Some(tx);

        is_decoding.store(true, Ordering::SeqCst);
        clock.set_sample_pos(0);

        let handle = thread::spawn(move || {
            while is_decoding.load(Ordering::Relaxed) {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        DecoderCommand::Seek(t) => {
                            decoder.seek(t);
                            producer.clear();
                        }
                        DecoderCommand::Stop => {
                            is_decoding.store(false, Ordering::SeqCst);
                            return;
                        }
                        DecoderCommand::SetBassBoost(v) => dsp.bass.set_enabled(v),
                        DecoderCommand::SetBassIntensity(v) => dsp.bass.set_intensity(v),
                    }
                }

                let rate = clock.get_sample_rate();
                let ch = clock.get_channels();

                if rate != output_rate || ch != output_channels {
                    output_rate = rate;
                    output_channels = ch;

                    resampler = if decoder_rate != output_rate || decoder_channels != output_channels as usize {
                        Some(Resampler::new(decoder_rate, output_rate, decoder_channels, 1024).unwrap())
                    } else {
                        None
                    };

                    dsp = DspChain::new(output_rate as f32, output_channels as usize);
                    dsp.bass.set_enabled(bass_boost_enabled.load(Ordering::SeqCst));
                    if let Ok(v) = bass_boost_intensity.lock() {
                        dsp.bass.set_intensity(*v);
                    }

                    producer.clear();
                }

                if producer.vacant_len() < 1024 {
                    thread::sleep(std::time::Duration::from_millis(5));
                    continue;
                }

                if let Some(mut samples) = decoder.decode_next() {
                    if let Some(r) = &mut resampler {
                        samples = r.process(&samples).unwrap_or(samples);
                    }

                    dsp.process(&mut samples);

                    let mut pushed = 0;
                    while pushed < samples.len() {
                        let n = producer.push_slice(&samples[pushed..]);
                        pushed += n;
                        if n == 0 {
                            thread::sleep(std::time::Duration::from_millis(2));
                        }
                    }
                } else {
                    if let Some(r) = &mut resampler {
                        if let Ok(flush) = r.flush() {
                            producer.push_slice(&flush);
                        }
                    }
                    is_decoding.store(false, Ordering::SeqCst);
                    return;
                }
            }
        });

        self.decode_thread = Some(handle);
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.clock.set_state(PlaybackState::Playing);
        self.output.start()?;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.clock.set_state(PlaybackState::Paused);
        self.output.pause()?;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.clock.set_state(PlaybackState::Stopped);
        let _ = self.output.stop();

        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(DecoderCommand::Stop);
        }

        self.is_decoding.store(false, Ordering::SeqCst);

        if let Some(h) = self.decode_thread.take() {
            let _ = h.join();
        }

        self.clock.set_sample_pos(0);
    }

    pub fn set_bass_boost(&self, enabled: bool) {
        self.bass_boost_enabled.store(enabled, Ordering::SeqCst);
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(DecoderCommand::SetBassBoost(enabled));
        }
    }

    pub fn set_bass_intensity(&self, intensity: f32) {
        if let Ok(mut v) = self.bass_boost_intensity.lock() {
            *v = intensity.clamp(0.0, 100.0);
        }
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(DecoderCommand::SetBassIntensity(intensity));
        }
    }

    pub fn seek(&mut self, time: f64) {
        let pos = (time * self.clock.get_sample_rate() as f64 * self.clock.get_channels() as f64) as u64;
        self.clock.set_sample_pos(pos);
        self.clock.signal_clear_buffer();
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(DecoderCommand::Seek(time));
        }
    }

    pub fn get_time_secs(&self) -> f64 {
        self.clock.get_time_secs()
    }

    pub fn tick(&mut self) {
        self.output.tick();
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

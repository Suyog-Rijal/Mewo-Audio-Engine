use std::path::Path;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};

use crate::engine::clock::{Clock, PlaybackState};
use crate::engine::decoder::{AudioDecoder, symphonia_decoder::SymphoniaDecoder};
use crate::engine::buffer::{create_audio_buffer, AudioBufferProducer};
use crate::engine::output::{AudioOutput, cpal_backend::CpalBackend, output_manager::OutputManager};
use crate::engine::dsp::resampler::Resampler;

enum DecoderCommand {
    Seek(f64),
    Stop,
}

pub struct AudioEngine {
    clock: Arc<Clock>,
    output: Box<dyn AudioOutput>,
    producer: Option<AudioBufferProducer>,
    decode_thread: Option<JoinHandle<()>>,
    is_decoding: Arc<AtomicBool>,
    command_tx: Option<Sender<DecoderCommand>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let clock = Arc::new(Clock::new(44100)); // Default, will be updated by output
        
        // Create buffer with a reasonable capacity (e.g., 1 second of stereo audio)
        let (producer, consumer) = create_audio_buffer(44100 * 2);
        
        let output = Box::new(OutputManager::new(consumer, clock.clone()));
        
        Ok(Self {
            clock,
            output,
            producer: Some(producer),
            decode_thread: None,
            is_decoding: Arc::new(AtomicBool::new(false)),
            command_tx: None,
        })
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();

        let mut decoder = SymphoniaDecoder::new(path)?;
        let mut producer = self.producer.take().ok_or("Producer already in use or missing")?;
        let is_decoding = self.is_decoding.clone();
        let clock = self.clock.clone();
        
        let mut output_sample_rate = self.clock.get_sample_rate();
        let mut output_channels = self.clock.get_channels();
        let decoder_sample_rate = decoder.sample_rate();
        let decoder_channels = decoder.channels() as usize;
        
        let mut resampler = if output_sample_rate != decoder_sample_rate || output_channels != decoder_channels as u32 {
            println!("Initializing resampler: {}Hz -> {}Hz, {}ch -> {}ch", decoder_sample_rate, output_sample_rate, decoder_channels, output_channels);
            Some(Resampler::new(decoder_sample_rate, output_sample_rate, decoder_channels, 1024)?)
        } else {
            None
        };
        
        let (tx, rx) = mpsc::channel();
        self.command_tx = Some(tx);
        
        is_decoding.store(true, Ordering::SeqCst);
        clock.set_sample_pos(0);
        
        let handle = thread::spawn(move || {
            while is_decoding.load(Ordering::Relaxed) {
                // Check for commands
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        DecoderCommand::Seek(time) => {
                            decoder.seek(time);
                            producer.clear();
                        }
                        DecoderCommand::Stop => {
                            is_decoding.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                }

                if !is_decoding.load(Ordering::Relaxed) {
                    break;
                }

                // Check if output sample rate or channels changed and we need to update resampler
                let current_output_rate = clock.get_sample_rate();
                let current_output_channels = clock.get_channels();
                if current_output_rate != output_sample_rate || current_output_channels != output_channels {
                    println!("Output config changed: {}Hz/{}ch -> {}Hz/{}ch. Reinitializing resampler.", 
                        output_sample_rate, output_channels, current_output_rate, current_output_channels);
                    
                    output_sample_rate = current_output_rate;
                    output_channels = current_output_channels;
                    
                    resampler = if output_sample_rate != decoder_sample_rate || output_channels != decoder_channels as u32 {
                        Some(Resampler::new(decoder_sample_rate, output_sample_rate, decoder_channels, 1024).unwrap())
                    } else {
                        None
                    };
                    
                    // Clear producer when output config changes to avoid glitches
                    producer.clear();
                }

                // If buffer is full, sleep briefly to avoid pegging CPU
                if producer.vacant_len() < 1024 {
                    thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }

                if let Some(samples) = decoder.decode_next() {
                    let processed_samples = if let Some(r) = &mut resampler {
                        r.process(&samples).unwrap_or_else(|e| {
                            eprintln!("Resampling error: {}", e);
                            samples // Fallback to original on error (not ideal)
                        })
                    } else {
                        samples
                    };

                    let mut pushed = 0;
                    while pushed < processed_samples.len() {
                        if !is_decoding.load(Ordering::Relaxed) {
                            break;
                        }
                        
                        // Check for commands even during pushing large chunks
                        if let Ok(cmd) = rx.try_recv() {
                             match cmd {
                                DecoderCommand::Seek(time) => {
                                    decoder.seek(time);
                                    producer.clear();
                                    break;
                                }
                                DecoderCommand::Stop => {
                                    is_decoding.store(false, Ordering::SeqCst);
                                    break;
                                }
                            }
                        }

                        let n = producer.push_slice(&processed_samples[pushed..]);
                        pushed += n;
                        if pushed < processed_samples.len() {
                            thread::sleep(std::time::Duration::from_millis(5));
                        }
                    }
                } else {
                    // EOF or Error
                    // Flush resampler if active
                    if let Some(r) = &mut resampler {
                        if let Ok(flushed) = r.flush() {
                            producer.push_slice(&flushed);
                        }
                    }
                    is_decoding.store(false, Ordering::SeqCst);
                    break;
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
        if let Some(handle) = self.decode_thread.take() {
            let _ = handle.join();
        }
        
        // Reset position
        self.clock.set_sample_pos(0);
    }

    pub fn seek(&mut self, time_secs: f64) {
        let sample_pos = (time_secs * self.clock.get_sample_rate() as f64 * self.clock.get_channels() as f64) as u64;
        self.clock.set_sample_pos(sample_pos);
        self.clock.signal_clear_buffer();
        
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(DecoderCommand::Seek(time_secs));
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

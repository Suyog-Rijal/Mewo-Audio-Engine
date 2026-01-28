use std::path::Path;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::engine::clock::{Clock, PlaybackState};
use crate::engine::decoder::{AudioDecoder, symphonia_decoder::SymphoniaDecoder};
use crate::engine::buffer::{create_audio_buffer, AudioBufferProducer};
use crate::engine::output::{AudioOutput, cpal_backend::CpalBackend};

pub struct AudioEngine {
    clock: Arc<Clock>,
    output: Box<dyn AudioOutput>,
    producer: Option<AudioBufferProducer>,
    decode_thread: Option<JoinHandle<()>>,
    is_decoding: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let clock = Arc::new(Clock::new(44100)); // Default, will be updated by output
        
        // Create buffer with a reasonable capacity (e.g., 1 second of stereo audio)
        let (producer, consumer) = create_audio_buffer(44100 * 2);
        
        let output = Box::new(CpalBackend::new(consumer, clock.clone())?);
        
        Ok(Self {
            clock,
            output,
            producer: Some(producer),
            decode_thread: None,
            is_decoding: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();

        let mut decoder = SymphoniaDecoder::new(path)?;
        let mut producer = self.producer.take().ok_or("Producer already in use or missing")?;
        let is_decoding = self.is_decoding.clone();
        let clock = self.clock.clone();
        
        is_decoding.store(true, Ordering::SeqCst);
        clock.set_sample_pos(0);
        
        let handle = thread::spawn(move || {
            while is_decoding.load(Ordering::Relaxed) {
                // If buffer is full, sleep briefly to avoid pegging CPU
                if producer.vacant_len() < 1024 {
                    thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }

                if let Some(samples) = decoder.decode_next() {
                    let mut pushed = 0;
                    while pushed < samples.len() {
                        if !is_decoding.load(Ordering::Relaxed) {
                            break;
                        }
                        let n = producer.push_slice(&samples[pushed..]);
                        pushed += n;
                        if pushed < samples.len() {
                            thread::sleep(std::time::Duration::from_millis(5));
                        }
                    }
                } else {
                    // EOF or Error
                    is_decoding.store(false, Ordering::SeqCst);
                    break;
                }
            }
            // In a real implementation, we'd want to return the producer back to the engine
            // or manage it differently so we can load new files.
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
        
        self.is_decoding.store(false, Ordering::SeqCst);
        if let Some(handle) = self.decode_thread.take() {
            let _ = handle.join();
        }
        
        // Reset position
        self.clock.set_sample_pos(0);
        
        // Note: The producer is currently lost after load() starts. 
        // A better design would use a channel to send it back or keep it in the engine.
    }

    pub fn seek(&mut self, time_secs: f64) {
        let sample_pos = (time_secs * self.clock.get_sample_rate() as f64) as u64;
        self.clock.set_sample_pos(sample_pos);
        // Note: Actual seeking in the decoder would require communication with the decode thread.
    }

    pub fn get_time_secs(&self) -> f64 {
        self.clock.get_time_secs()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

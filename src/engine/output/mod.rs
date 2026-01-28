pub mod cpal_backend;
pub mod output_manager;

use crate::engine::buffer::AudioBufferConsumer;
use crate::engine::clock::Clock;
use std::sync::Arc;

pub trait AudioOutput {
    /// Starts the audio output stream.
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Pauses the audio output stream.
    fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Stops the audio output stream.
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Checks if the output is still healthy.
    fn is_healthy(&self) -> bool;

    /// Shutdown the backend and return the consumer if possible.
    fn shutdown(&mut self) -> Option<AudioBufferConsumer>;

    /// Periodically check for device changes or health issues.
    fn tick(&mut self);
}

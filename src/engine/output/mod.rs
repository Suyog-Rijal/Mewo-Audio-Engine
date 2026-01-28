pub mod cpal_backend;

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
}

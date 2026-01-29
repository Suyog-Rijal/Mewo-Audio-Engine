pub mod cpal_backend;
pub mod output_manager;

use crate::engine::buffer::AudioBufferConsumer;
use crate::engine::clock::Clock;
use std::sync::Arc;

pub trait AudioOutput: Send {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn is_healthy(&self) -> bool;
    fn shutdown(&mut self) -> Option<AudioBufferConsumer>;
    fn tick(&mut self);
    fn clear_buffer(&mut self);
}
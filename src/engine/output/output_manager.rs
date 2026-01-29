use std::sync::Arc;
use crate::engine::buffer::AudioBufferConsumer;
use crate::engine::clock::{Clock, PlaybackState};
use crate::engine::output::cpal_backend::CpalBackend;
use crate::engine::output::AudioOutput;

pub struct OutputManager {
    backend: Option<CpalBackend>,
    consumer: Option<AudioBufferConsumer>,
    clock: Arc<Clock>,
}

impl OutputManager {
    pub fn new(consumer: AudioBufferConsumer, clock: Arc<Clock>) -> Self {
        let mut manager = Self {
            backend: None,
            consumer: Some(consumer),
            clock,
        };
        let _ = manager.try_reconnect();
        manager
    }

    pub fn try_reconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(consumer) = self.consumer.take() {
            match CpalBackend::new(consumer, self.clock.clone()) {
                Ok(backend) => {
                    self.backend = Some(backend);
                    Ok(())
                }
                Err((recovered_consumer, e)) => {
                    self.consumer = Some(recovered_consumer);
                    eprintln!("Failed to reconnect audio: {}", e);
                    Err(e)
                }
            }
        } else {
            Err("Consumer missing".into())
        }
    }

    pub fn check_connection(&mut self) {
        let needs_reconnect = match &self.backend {
            Some(backend) => !backend.is_healthy(),
            None => true,
        };

        if needs_reconnect {
            let previous_state = self.clock.get_state();
            if let Some(mut backend) = self.backend.take() {
                if let Some(consumer) = backend.shutdown() {
                    self.consumer = Some(consumer);
                }
            }
            if self.try_reconnect().is_ok() {
                if previous_state == PlaybackState::Playing {
                    let _ = self.start();
                }
            }
        }
    }
}

impl AudioOutput for OutputManager {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.check_connection();
        if let Some(backend) = &mut self.backend {
            backend.start()
        } else {
            Err("No audio backend available".into())
        }
    }

    fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(backend) = &mut self.backend {
            backend.pause()
        } else {
            Ok(())
        }
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(backend) = &mut self.backend {
            backend.stop()
        } else {
            Ok(())
        }
    }

    fn is_healthy(&self) -> bool {
        match &self.backend {
            Some(backend) => backend.is_healthy(),
            None => false,
        }
    }

    fn shutdown(&mut self) -> Option<AudioBufferConsumer> {
        if let Some(mut backend) = self.backend.take() {
            backend.shutdown().or(self.consumer.take())
        } else {
            self.consumer.take()
        }
    }

    fn tick(&mut self) {
        self.check_connection();
    }
}
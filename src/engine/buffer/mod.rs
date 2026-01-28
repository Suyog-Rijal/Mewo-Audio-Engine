use std::sync::Arc;
use ringbuf::{
    traits::{Consumer, Producer, Split, Observer},
    HeapRb,
    CachingProd,
    CachingCons,
};

/// The Buffer subsystem handles the thread-safe transport of audio data.
/// It uses a lock-free Single-Producer Single-Consumer (SPSC) ring buffer.
pub struct AudioBuffer {
    // This could hold configuration or the initial RB before splitting
}

/// Producer handle for the audio buffer. Used by the Decoder.
pub struct AudioBufferProducer {
    inner: CachingProd<Arc<HeapRb<f32>>>,
}

/// Consumer handle for the audio buffer. Used by the Output.
pub struct AudioBufferConsumer {
    inner: CachingCons<Arc<HeapRb<f32>>>,
}

impl AudioBufferProducer {
    /// Pushes a single sample into the buffer.
    /// Returns Err if the buffer is full.
    pub fn push(&mut self, sample: f32) -> Result<(), f32> {
        self.inner.try_push(sample)
    }

    /// Pushes a slice of samples into the buffer.
    /// Returns the number of samples successfully pushed.
    pub fn push_slice(&mut self, samples: &[f32]) -> usize {
        self.inner.push_slice(samples)
    }

    /// Returns the number of free spaces in the buffer.
    pub fn vacant_len(&self) -> usize {
        self.inner.vacant_len()
    }
}

impl AudioBufferConsumer {
    /// Pops a single sample from the buffer.
    /// Returns None if the buffer is empty.
    pub fn pop(&mut self) -> Option<f32> {
        self.inner.try_pop()
    }

    /// Pops samples into the provided slice.
    /// Returns the number of samples successfully popped.
    pub fn pop_slice(&mut self, samples: &mut [f32]) -> usize {
        self.inner.pop_slice(samples)
    }

    /// Returns the number of samples available in the buffer.
    pub fn occupied_len(&self) -> usize {
        self.inner.occupied_len()
    }
}

/// Creates a new audio buffer with the specified capacity.
/// Returns a (Producer, Consumer) pair.
pub fn create_audio_buffer(capacity: usize) -> (AudioBufferProducer, AudioBufferConsumer) {
    let rb = HeapRb::<f32>::new(capacity);
    let (prod, cons) = rb.split();
    (
        AudioBufferProducer { inner: prod },
        AudioBufferConsumer { inner: cons },
    )
}

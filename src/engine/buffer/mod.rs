use std::sync::Arc;
use ringbuf::{traits::{Consumer, Producer, Split, Observer}, HeapRb, CachingProd, CachingCons};

pub struct AudioBuffer {}

pub struct AudioBufferProducer {
    inner: CachingProd<Arc<HeapRb<f32>>>,
}

pub struct AudioBufferConsumer {
    inner: CachingCons<Arc<HeapRb<f32>>>,
}

impl AudioBufferProducer {
    pub fn push(&mut self, sample: f32) -> Result<(), f32> {
        self.inner.try_push(sample)
    }

    pub fn push_slice(&mut self, samples: &[f32]) -> usize {
        self.inner.push_slice(samples)
    }

    pub fn vacant_len(&self) -> usize {
        self.inner.vacant_len()
    }

    pub fn clear(&mut self) {}
}

impl AudioBufferConsumer {
    pub fn pop(&mut self) -> Option<f32> {
        self.inner.try_pop()
    }

    pub fn pop_slice(&mut self, samples: &mut [f32]) -> usize {
        self.inner.pop_slice(samples)
    }

    pub fn occupied_len(&self) -> usize {
        self.inner.occupied_len()
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn empty() -> Self {
        let rb = HeapRb::<f32>::new(1);
        let (_, cons) = rb.split();
        AudioBufferConsumer { inner: cons }
    }
}

pub fn create_audio_buffer(capacity: usize) -> (AudioBufferProducer, AudioBufferConsumer) {
    let rb = HeapRb::<f32>::new(capacity);
    let (prod, cons) = rb.split();
    (AudioBufferProducer { inner: prod }, AudioBufferConsumer { inner: cons })
}
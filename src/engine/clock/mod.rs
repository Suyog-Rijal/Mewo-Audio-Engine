use std::sync::atomic::{AtomicU64, AtomicU8, AtomicBool, Ordering};

/// Represents the current playback state of the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlaybackState {
    Stopped = 0,
    Playing = 1,
    Paused = 2,
}

impl From<u8> for PlaybackState {
    fn from(value: u8) -> Self {
        match value {
            1 => PlaybackState::Playing,
            2 => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        }
    }
}

/// The Clock is the global timing authority of the audio engine.
/// It maintains the playback position and state using atomic variables
/// to ensure real-time safety and thread-safe access.
pub struct Clock {
    /// Current playback position in samples.
    sample_pos: AtomicU64,
    /// Current sample rate (e.g., 44100, 48000).
    sample_rate: AtomicU64,
    /// Current number of channels.
    channels: AtomicU8,
    /// Current state of playback (Stored as u8 for atomicity).
    state: AtomicU8,
    /// Flag to signal the buffer should be cleared.
    clear_buffer: AtomicBool,
}

impl Clock {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_pos: AtomicU64::new(0),
            sample_rate: AtomicU64::new(sample_rate as u64),
            channels: AtomicU8::new(2),
            state: AtomicU8::new(PlaybackState::Stopped as u8),
            clear_buffer: AtomicBool::new(false),
        }
    }

    /// Returns the current playback position in samples.
    pub fn get_sample_pos(&self) -> u64 {
        self.sample_pos.load(Ordering::Relaxed)
    }

    /// Sets the current playback position in samples (used for seeking).
    pub fn set_sample_pos(&self, pos: u64) {
        self.sample_pos.store(pos, Ordering::SeqCst);
    }

    /// Increments the sample position by a given amount.
    /// Typically called by the output layer after processing a block.
    pub fn increment_samples(&self, amount: u64) {
        if self.get_state() == PlaybackState::Playing {
            self.sample_pos.fetch_add(amount, Ordering::Relaxed);
        }
    }

    /// Returns the current playback position in seconds.
    pub fn get_time_secs(&self) -> f64 {
        let pos = self.get_sample_pos() as f64;
        let rate = self.sample_rate.load(Ordering::Relaxed) as f64;
        let channels = self.get_channels() as f64;
        if rate > 0.0 && channels > 0.0 {
            pos / (rate * channels)
        } else {
            0.0
        }
    }

    /// Returns the current playback state.
    pub fn get_state(&self) -> PlaybackState {
        PlaybackState::from(self.state.load(Ordering::Relaxed))
    }

    /// Sets the playback state.
    pub fn set_state(&self, state: PlaybackState) {
        self.state.store(state as u8, Ordering::SeqCst);
    }

    /// Updates the sample rate.
    pub fn set_sample_rate(&self, rate: u32) {
        self.sample_rate.store(rate as u64, Ordering::SeqCst);
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed) as u32
    }

    pub fn set_channels(&self, channels: u32) {
        self.channels.store(channels as u8, Ordering::SeqCst);
    }

    pub fn get_channels(&self) -> u32 {
        self.channels.load(Ordering::Relaxed) as u32
    }

    pub fn signal_clear_buffer(&self) {
        self.clear_buffer.store(true, Ordering::SeqCst);
    }

    pub fn should_clear_buffer(&self) -> bool {
        self.clear_buffer.load(Ordering::Relaxed)
    }

    pub fn reset_clear_buffer(&self) {
        self.clear_buffer.store(false, Ordering::SeqCst);
    }
}

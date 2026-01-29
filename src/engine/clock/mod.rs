use std::sync::atomic::{AtomicU64, AtomicU8, AtomicBool, Ordering};

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

pub struct Clock {
    sample_pos: AtomicU64,
    sample_rate: AtomicU64,
    channels: AtomicU8,
    state: AtomicU8,
    clear_buffer: AtomicBool,
    eos: AtomicBool,
}

impl Clock {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_pos: AtomicU64::new(0),
            sample_rate: AtomicU64::new(sample_rate as u64),
            channels: AtomicU8::new(2),
            state: AtomicU8::new(PlaybackState::Stopped as u8),
            clear_buffer: AtomicBool::new(false),
            eos: AtomicBool::new(false),
        }
    }

    pub fn get_sample_pos(&self) -> u64 {
        self.sample_pos.load(Ordering::Relaxed)
    }

    pub fn set_sample_pos(&self, pos: u64) {
        self.sample_pos.store(pos, Ordering::SeqCst);
    }

    pub fn increment_samples(&self, amount: u64) {
        if self.get_state() == PlaybackState::Playing {
            self.sample_pos.fetch_add(amount, Ordering::Relaxed);
        }
    }

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

    pub fn get_state(&self) -> PlaybackState {
        PlaybackState::from(self.state.load(Ordering::Relaxed))
    }

    pub fn set_state(&self, state: PlaybackState) {
        self.state.store(state as u8, Ordering::SeqCst);
    }

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

    pub fn set_eos(&self, eos: bool) {
        self.eos.store(eos, Ordering::SeqCst);
    }

    pub fn is_eos(&self) -> bool {
        self.eos.load(Ordering::Relaxed)
    }
}
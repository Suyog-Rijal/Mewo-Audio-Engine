pub mod symphonia_decoder;

pub trait AudioDecoder {
    /// Decodes the next block of audio data and returns it as a Vec<f32>.
    /// Returns None when the end of the stream is reached.
    fn decode_next(&mut self) -> Option<Vec<f32>>;

    /// Returns the sample rate of the audio.
    fn sample_rate(&self) -> u32;

    /// Returns the number of channels.
    fn channels(&self) -> u32;

    /// Seeks to a specific time in seconds.
    fn seek(&mut self, time_secs: f64);

    /// Returns the total duration of the audio in seconds, if known.
    fn duration(&self) -> Option<f64>;
}

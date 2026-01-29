pub mod symphonia_decoder;

#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    pub duration_secs: Option<f64>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}

pub trait AudioDecoder {
    fn decode_next(&mut self) -> Option<Vec<f32>>;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u32;
    fn seek(&mut self, time_secs: f64);
    fn duration(&self) -> Option<f64>;
    fn metadata(&self) -> Option<AudioMetadata>;
}
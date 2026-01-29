pub mod symphonia_decoder;

pub trait AudioDecoder {
    fn decode_next(&mut self) -> Option<Vec<f32>>;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u32;
    fn seek(&mut self, time_secs: f64);
    fn duration(&self) -> Option<f64>;
}
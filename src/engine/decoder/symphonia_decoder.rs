use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use crate::engine::decoder::AudioDecoder;

pub struct SymphoniaDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: u32,
    duration: Option<f64>,
}

impl SymphoniaDecoder {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path_ref.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();
        let dec_opts = DecoderOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)?;

        let reader = probed.format;

        let track = reader.tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or("No supported audio tracks found")?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count() as u32).unwrap_or(2);

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)?;

        let duration = track.codec_params.n_frames.map(|frames| {
            frames as f64 / sample_rate as f64
        });

        Ok(Self {
            reader,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration,
        })
    }
}

impl AudioDecoder for SymphoniaDecoder {
    fn decode_next(&mut self) -> Option<Vec<f32>> {
        loop {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => return None,
                Err(err) => {
                    eprintln!("Decoder error: {:?}", err);
                    return None;
                }
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(audio_buf) => {
                    let spec = *audio_buf.spec();
                    let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.capacity() as u64, spec);
                    sample_buf.copy_interleaved_ref(audio_buf);
                    return Some(sample_buf.samples().to_vec());
                }
                Err(Error::DecodeError(err)) => {
                    eprintln!("Decode error: {:?}", err);
                    continue;
                }
                Err(err) => {
                    eprintln!("Unexpected decoder error: {:?}", err);
                    return None;
                }
            }
        }
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u32 {
        self.channels
    }

    fn seek(&mut self, time_secs: f64) {
        let _ = self.reader.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time: Time::from(time_secs),
                track_id: Some(self.track_id),
            },
        );
    }

    fn duration(&self) -> Option<f64> {
        self.duration
    }
}

use std::path::{Path, PathBuf};
use anyhow::Result;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};
use std::fs::File;

pub struct Audio {
    path: PathBuf,
    pcm: Vec<i16>,
}

impl Audio {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn pcm(&self) -> &[i16] {
        &self.pcm
    }

    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path_buf = path.into();
        let file = File::open(&path_buf)?;
        let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

        let probed = get_probe().format(
            &Hint::default(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;
        let mut format = probed.format;

        let track = format.default_track().ok_or_else(|| anyhow::anyhow!("no default track"))?;
        let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

        let mut pcm = Vec::new();

        while let Ok(packet) = format.next_packet() {
            let decoded = decoder.decode(&packet)?;
            let mut buf = SampleBuffer::<i16>::new(decoded.frames().try_into().unwrap(), *decoded.spec());
            buf.copy_interleaved_ref(decoded);
            pcm.extend_from_slice(buf.samples());
        }

        Ok(Self { path: path_buf, pcm })

    }

    pub fn mean_absolute(&self) -> f64 {
        let sum: i64 = self.pcm.iter().map(|&s| i64::from(s).abs()).sum();
        sum as f64 / self.pcm.len() as f64
    }
}
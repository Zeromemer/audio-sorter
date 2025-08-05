use core::{
    error::Error,
    fmt::{self, Display},
};
use std::{
    io::Error as IoError, path::{Path, PathBuf}, process::{Command, ExitStatus, Stdio}
};

use itertools::Itertools;

pub struct Audio {
    path: PathBuf,
    pcm: Vec<i16>,
}

#[derive(Debug)]
pub enum AudioExtractError {
    Command(IoError),
    FFmpeg(ExitStatus, String),
}

impl Display for AudioExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(err) => write!(f, "Error instantiating command {err}")?,
            Self::FFmpeg(status, err) => write!(f, "FFmpeg: {err} (status code {status})")?,
        }
        Ok(())
    }
}

impl Error for AudioExtractError {}

impl Audio {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn pcm(&self) -> &Vec<i16> {
        &self.pcm
    }

    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self, AudioExtractError> {
        let path = path.into();

        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(&path)
            .arg("-f")
            .arg("s16le") // Raw PCM 16-bit
            .arg("-ac")
            .arg("1") // Mono
            .arg("-ar")
            .arg("44100") // 44.1kHz
            .arg("pipe:1") // Output to stdout
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(AudioExtractError::Command)?;

        if ffmpeg.status.success() {
            let pcm = ffmpeg
                .stdout
                .iter()
                .tuples()
                .map(|(&a, &b)| i16::from_le_bytes([a, b]))
                .collect();

            Ok(Self { path, pcm })
        } else {
            Err(AudioExtractError::FFmpeg(
                ffmpeg.status,
                String::from_utf8(ffmpeg.stderr).unwrap(),
            ))
        }
    }

    pub fn mean_absolute(&self) -> f64 {
        let sum: i64 = self.pcm.iter().map(|&s| i64::from(s).abs()).sum();
        sum as f64 / self.pcm.len() as f64
    }
}

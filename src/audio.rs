use std::path::{Path, PathBuf};
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_audio::AudioFormat;
use itertools::Itertools;
use anyhow::Result;

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
        let path = path.into();

        let filesrc = gst::ElementFactory::make("filesrc").build()?;
        let path_str = path.to_string_lossy();
        filesrc.set_property("location", path_str.as_ref());

        let decodebin = gst::ElementFactory::make("decodebin").build()?;
        let convert = gst::ElementFactory::make("audioconvert").build()?;
        let resample = gst::ElementFactory::make("audioresample").build()?;

        let capsfilter = gst::ElementFactory::make("capsfilter").build()?;
        let caps = gst::Caps::builder("audio/x-raw")
           .field("format", AudioFormat::S16le.to_str())
           .field("channels", 1i32)
           .field("rate", 44100i32)
           .build();
        capsfilter.set_property("caps", &caps);

        // Create the appsink element as a generic gst::Element first
        let appsink_element = gst::ElementFactory::make("appsink").build()?;

        // Downcast to AppSink to access its specific methods
        let appsink_specific = appsink_element.clone()
           .downcast::<AppSink>().expect("Failed to downcast to AppSink");

        // Now use appsink_specific for AppSink-specific property settings
        appsink_specific.set_caps(Some(&caps));
        appsink_specific.set_property("sync", false);
        appsink_specific.set_property("emit-signals", false);

        let pipeline = gst::Pipeline::new();
        pipeline.add_many([&filesrc, &decodebin, &convert, &resample, &capsfilter, &appsink_element])?;

        gst::Element::link(&filesrc, &decodebin)?;

        // connect dynamic pad from decodebin to convert
        let convert_clone = convert.clone();
        decodebin.connect_pad_added(move |_, src_pad| {
            if let Some(sink_pad) = convert_clone.static_pad("sink") {
                let _ = src_pad.link(&sink_pad);
            }
        });

        gst::Element::link_many([&convert, &resample, &capsfilter, &appsink_element])?;

        pipeline.set_state(gst::State::Playing)?;

        // Pull samples using appsink_specific
        let mut pcm = Vec::new();
        while let Ok(sample) = appsink_specific.pull_sample() {
            if let Some(buffer) = sample.buffer() {
                if let Ok(map) = buffer.map_readable() {
                    let shorts = map.iter().tuples().map(|(a, b)| i16::from_le_bytes([*a, *b]));
                    pcm.extend(shorts);
                }
            }
        }

        pipeline.set_state(gst::State::Null)?;
        Ok(Self { path, pcm })
    }

    pub fn mean_absolute(&self) -> f64 {
        let sum: i64 = self.pcm.iter().map(|&s| i64::from(s).abs()).sum();
        sum as f64 / self.pcm.len() as f64
    }
}
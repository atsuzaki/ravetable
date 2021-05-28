// TODO: write title for this thing
//! Clipped Sine Wave assignment
//!
//! Playback code largely adapted from cpal examples: https://github.com/RustAudio/cpal/tree/master/examples
//! Output WAV is saved to "$CARGO_MANIFEST_DIR/recorded.wav".
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

#![feature(format_args_capture)]

use hound;
use log::{error, info, warn};
use std::thread;

use crate::playback::run;
use crate::utils::sample_converter::load_waveform;
use cpal::traits::{DeviceTrait, HostTrait};
use hound::WavSpec;

mod gui;
mod playback;
mod synths;
mod utils;

pub struct InputWav {
    samples: Option<Vec<f32>>,
    spec: WavSpec,
    file_path: String,
}

fn main() -> Result<(), anyhow::Error> {
    init_logger();

    let test_wav = "test_wavs/CantinaBand.wav".to_string();

    let host = cpal::default_host();

    let device = host.default_output_device().expect("Device failed");
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);

    let input_wav = load_waveform(test_wav, config.sample_rate().0);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), input_wav),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), input_wav),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), input_wav),
    };

    Ok(())
}

#[derive(Debug)]
struct Opt {
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    jack: bool,

    device: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Note(f32),
    Frequency(f32),
    Amplitude(f32),
}

pub type CrossbeamReceiver = crossbeam_channel::Receiver<Message>;
pub type CrossbeamSender = crossbeam_channel::Sender<Message>;

fn init_logger() {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Warn)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        //.chain(fern::log_file(&log_filepath).unwrap())
        // Apply globally
        .apply()
        .unwrap();
}

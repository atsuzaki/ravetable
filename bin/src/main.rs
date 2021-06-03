// TODO: write title for this thing
//! Ravetable
//!
//! Playback code largely adapted from cpal examples: https://github.com/RustAudio/cpal/tree/master/examples
//! Output WAV is saved to "$CARGO_MANIFEST_DIR/recorded.wav".
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

#![feature(format_args_capture)]

use cpal::traits::{DeviceTrait, HostTrait};
use once_cell::sync::OnceCell;

use crate::mixer::Mixer;
use crate::playback::run;
use crate::synths::{Oscillator, Wavetable};
use cpal::SampleRate;

mod gui;
mod mixer;
mod playback;
mod synths;
mod utils;

pub static SAMPLE_RATE: OnceCell<SampleRate> = OnceCell::new();

pub fn get_sample_rate() -> f32 {
    SAMPLE_RATE.get().unwrap().0 as f32
}

fn main() -> Result<(), anyhow::Error> {
    init_logger();

    let host = cpal::default_host();

    let device = host.default_output_device().expect("Device failed");
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);
    SAMPLE_RATE.set(config.sample_rate()).unwrap();

    let wavetable = Wavetable::create_wavetable(
        "test_wavs/CantinaBandMONO.wav".to_string(),
        config.sample_rate().0,
    );
    let osc = Oscillator::new(0.65, wavetable);

    let wavetable2 =
        Wavetable::create_wavetable("test_wavs/sine.wav".to_string(), config.sample_rate().0);
    let osc2 = Oscillator::new(0.10, wavetable2);

    let mixer = Mixer::new(vec![osc, osc2]);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), mixer),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), mixer),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), mixer),
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

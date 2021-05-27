// TODO: write title for this thing
//! Clipped Sine Wave assignment
//!
//! Playback code largely adapted from cpal examples: https://github.com/RustAudio/cpal/tree/master/examples
//! Output WAV is saved to "$CARGO_MANIFEST_DIR/recorded.wav".
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

//use rodio::{Decoder, OutputStream, source::Source};
use hound;
use log::{info, warn};
use std::thread;

use std::fs::File;
use std::io::{BufReader, Cursor};
use tuix::Entity;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

mod utils;
mod playback;
mod gui;

fn main() -> Result<(), anyhow::Error> {
	init_logger();

	//// Get a output stream handle to the default physical sound device
	//let (_stream, stream_handle) = OutputStream::try_default().unwrap();
	//let sink = rodio::Sink::try_new(&stream_handle).unwrap();
	//
	//// Load a sound from a file, using a path relative to Cargo.toml
	//let file = BufReader::new(File::open("test_wavs/CantinaBand.wav").unwrap());

	let mut reader = hound::WavReader::open("test_wavs/CantinaBand.wav").unwrap();

	let mut samples = reader.samples::<f32>();
	let fsamples: Vec<f32> = samples.map(|f| f.unwrap()).collect();

	//hound done, we have f32 samples now
	//start cpal playback and init and shit

	let host = cpal::default_host();

	let device = host.default_output_device().expect("Device failed");
	println!("Output device: {}", device.name()?);

	let config = device.default_output_config().unwrap();
	println!("Default output config: {:?}", config);

	match config.sample_format() {
		cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), fsamples),
		cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), fsamples),
		cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), fsamples),
	};

	Ok(())
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, mut samples: Vec<f32>) -> Result<(), anyhow::Error>
	where
		T: cpal::Sample,
{
	let sample_rate = config.sample_rate.0 as f32;
	let channels = config.channels as usize;

	// Produce a sinusoid of maximum amplitude.
	let mut sample_clock = 0f32;
	let mut next_value = move || {
		//sample_clock = (sample_clock + 1.0) % sample_rate;
		//(sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
		samples.pop().unwrap()
	};

	let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

	let stream = device.build_output_stream(
		config,
		move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
			write_data(data, channels, &mut next_value)
		},
		err_fn,
	)?;
	stream.play()?;

	std::thread::sleep(std::time::Duration::from_millis(100000));

	Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
	where
		T: cpal::Sample,
{
	for frame in output.chunks_mut(channels) {
		let value: T = cpal::Sample::from::<f32>(&next_sample());
		for sample in frame.iter_mut() {
			*sample = value;
		}
	}
}

pub struct Oscillator {
	pub phi: f32,
	pub frequency: f32,
	pub amplitude: f32,
	pub enabled: bool,
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
				message))
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

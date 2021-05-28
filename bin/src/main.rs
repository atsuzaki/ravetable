// TODO: write title for this thing
//! Clipped Sine Wave assignment
//!
//! Playback code largely adapted from cpal examples: https://github.com/RustAudio/cpal/tree/master/examples
//! Output WAV is saved to "$CARGO_MANIFEST_DIR/recorded.wav".
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

//use rodio::{Decoder, OutputStream, source::Source};
use hound;
use log::{info, warn, error};
use std::thread;

use std::fs::File;
use std::io::{BufReader, Cursor};
use tuix::Entity;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use hound::WavSpec;
use cpal::SampleRate;
use samplerate::{Samplerate, ConverterType};

mod utils;
mod playback;
mod gui;

pub struct InputWav {
	samples: Option<Vec<f32>>,
	spec: WavSpec,
	file_path: String,
}

fn main() -> Result<(), anyhow::Error> {
	init_logger();

	let test_wav = "test_wavs/CantinaBand44100.wav".to_string();

	let mut reader = hound::WavReader::open(&test_wav).unwrap();
	let input_wav_spec = reader.spec();


	let mut samples = reader.into_samples::<f32>();
	let mut fsamples: Vec<f32> = samples.map(|f| f.unwrap()).collect();
	fsamples.reverse();

	//hound done, we have f32 samples now
	//start cpal playback and init and shit

	let host = cpal::default_host();

	let device = host.default_output_device().expect("Device failed");
	println!("Output device: {}", device.name()?);

	let config = device.default_output_config().unwrap();
	println!("Default output config: {:?}", config);

	if input_wav_spec.sample_rate != config.sample_rate().0	{
		println!("Converting sample");
		// Instanciate a new converter.
		let mut sample_rate_converter = Samplerate::new(
			ConverterType::SincBestQuality,
			input_wav_spec.sample_rate,
			config.sample_rate().0,
			input_wav_spec.channels as usize)
			.unwrap();


		// Resample the input from input sample rate to output sample rate
		fsamples = sample_rate_converter.process_last(&fsamples).unwrap();
	}

	let mut input_wav = InputWav {
		samples: Some(fsamples),
		spec: input_wav_spec,
		file_path: test_wav,
	};

	match config.sample_format() {
		cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), input_wav),
		cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), input_wav),
		cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), input_wav),
	};

	Ok(())
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, mut input_wav: InputWav) -> Result<(), anyhow::Error>
	where
		T: cpal::Sample,
{
	let output_sample_rate = config.sample_rate.0 as f32;
	let output_channels = config.channels as usize;
	let input_channels = input_wav.spec.channels;
	let mut samples = input_wav.samples.take().unwrap(); //input_wav's job is basically done now

	let mut next_value = move || {
		samples.pop().unwrap()
	};

	let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

	println!("Channels for output: {}", output_channels);

	let stream = device.build_output_stream(
		config,
		move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
			write_data(data, output_channels, input_channels, &mut next_value)
		},
		err_fn,
	)?;
	stream.play()?;

	std::thread::sleep(std::time::Duration::from_millis(100000));

	Ok(())
}

fn write_data<T>(output: &mut [T], output_channels: usize, input_channels: u16, next_sample: &mut dyn FnMut() -> f32)
	where
		T: cpal::Sample,
{
	for frame in output.chunks_mut(output_channels) {
		match input_channels {
			1 => {
				let value: T = cpal::Sample::from::<f32>(&next_sample());
				for sample in frame.iter_mut() {
					*sample = value;
				}
			}
			2 => {
				for sample in frame.iter_mut() {
					*sample = cpal::Sample::from::<f32>(&next_sample());
				}
			}
			_ => panic!("Unsupported channels found in input audio")
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

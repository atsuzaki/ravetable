// TODO: write title for this thing
//! Clipped Sine Wave assignment
//!
//! Playback code largely adapted from cpal examples: https://github.com/RustAudio/cpal/tree/master/examples
//! Output WAV is saved to "$CARGO_MANIFEST_DIR/recorded.wav".
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

use rodio::{Decoder, OutputStream, source::Source};
use log::{info, warn};
use std::thread;

use std::fs::File;
use std::io::{BufReader, Cursor};
use tuix::Entity;

mod utils;
mod playback;
mod gui;

fn main() -> Result<(), anyhow::Error> {
	init_logger();

	// Get a output stream handle to the default physical sound device
	let (_stream, stream_handle) = OutputStream::try_default().unwrap();
	let sink = rodio::Sink::try_new(&stream_handle).unwrap();

	// Load a sound from a file, using a path relative to Cargo.toml
	let file = BufReader::new(File::open("test_wavs/CantinaBand.wav").unwrap());

	// Decode that sound file into a source
	let mut decoder = Decoder::new(file).unwrap();
	let channels = decoder.channels();
	let sample_rate = decoder.sample_rate();
	let source = decoder.collect::<Vec<f32>>(); // decode the full song

	//make it quiet just to test that editing source samples works
	let modified_source: Vec<f32> = source.iter().map(|&i| (i as f32 * 0.25) as f32).collect();

	//info!("{:?}", &modified_source);

	//feed the new source
	sink.append(rodio::buffer::SamplesBuffer::new(channels, sample_rate, modified_source));

	sink.sleep_until_end();

	Ok(())
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

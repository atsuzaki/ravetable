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

use crate::playback::run;
use crate::synths::{Oscillator, Wavetable};
use cpal::SampleRate;
use crate::mixer::Mixer;
use tuix::{Application, Button, Widget};
use tuix::*;
use tuix::state::themes::DEFAULT_THEME;
use crate::gui::Controller;

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

	//setup_tuix();

    let host = cpal::default_host();

    let device = host.default_output_device().expect("Device failed");
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);
    SAMPLE_RATE.set(config.sample_rate()).unwrap();

    let wavetable = Wavetable::create_wavetable("test_wavs/CantinaBandMONO.wav".to_string(), config.sample_rate().0);
    let osc = Oscillator::new(0.65, wavetable);

    let wavetable2 = Wavetable::create_wavetable("test_wavs/sine.wav".to_string(), config.sample_rate().0);
    let osc2 = Oscillator::new(0.10, wavetable2);

    let mixer = Mixer::new(vec![osc, osc2]);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), mixer),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), mixer),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), mixer),
    };

    Ok(())
}

fn setup_tuix() {
    let (command_sender, command_receiver) = crossbeam_channel::bounded(1024);

    //this is my tuix example from my tuix repo
	let app = Application::new(|state, window| {
		match state.add_stylesheet("bin/src/bbytheme.css") {
			Ok(_) => {}
			Err(e) => println!("Error loading stylesheet: {}", e),
		}

		window
			.set_title("basic")
			.set_background_color(state, Color::rgb(55, 255, 255))
			.set_align_items(state, AlignItems::FlexStart);

        let controller = Controller::new(command_sender.clone()).build(state, window.entity(), |builder| builder);

		let _one = Element::new().build(state, window.entity(), |builder| {
			builder
				.class("one")
				.set_background_gradient(
					LinearGradient::new(Direction::TopToBottom)
						.add_stop(GradientStop::new(
							Units::Pixels(0.0),
							Color::rgb(190, 90, 190),
						))
						.add_stop(GradientStop::new(
							Units::Pixels(30.0),
							Color::rgb(50, 50, 50),
						)),
				)
				.set_text("Button")
		});

	});

	app.run();


    ///////////// TODO: THIS IS THE CODE WE HAD IN BBYS_SYNTH
    //let app = Application::new(|win_desc, state, window| {
    //    state.style.parse_theme(THEME);
	//
    //    Controller::new(command_sender.clone()).build(state, window, |builder| builder);
    //    win_desc.with_title("BbySynth").with_inner_size(200, 200)
    //});
    // app.run();
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

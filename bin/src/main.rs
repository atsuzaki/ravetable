// TODO: write title for this thing
//! Ravetable
//!
//! Playback code largely adapted from cpal examples: https://github.com/RustAudio/cpal/tree/master/examples
//! Output WAV is saved to "$CARGO_MANIFEST_DIR/recorded.wav".
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

//#![feature(format_args_capture)]

use cpal::traits::{DeviceTrait, HostTrait};

use crate::gui::Controller;
use crate::mixer::Mixer;
use crate::playback::run;
use crate::synths::{Oscillator, Wavetable};
use std::thread;
use tuix::state::themes::DEFAULT_THEME;
use tuix::*;

use crate::keyboard::MidiKeyboard;
use crate::state::{get_sample_rate, set_sample_rate};
use effects::filters::{
    Filter, FilterType, IIRLowPassFilter, ModulatedFilter, StateVariableTPTFilter,
};
use effects::lfo::{Lfo, LfoType};
use effects::{set_effects_sample_rate, Effect};

mod gui;
mod keyboard;
mod mixer;
mod playback;
mod state;
mod synths;
mod utils;

fn main() -> Result<(), anyhow::Error> {
    let (command_sender, command_receiver) = crossbeam_channel::bounded(1024);

    init_logger();

    // Audio backend must be started first, as GUI runs on main threadbecause of OSX
    start_audio_backend(command_receiver);

    start_gui(command_sender);

    Ok(())
}

fn start_audio_backend(command_receiver: crossbeam_channel::Receiver<Message>) {
    thread::spawn(|| {
        let host = cpal::default_host();

        let device = host.default_output_device().expect("Device failed");
        println!(
            "Output device: {}",
            device.name().expect("No output device found")
        );

        let config = device.default_output_config().unwrap();
        println!("Default output config: {:?}", config);
        let sample_rate = config.sample_rate();

        set_sample_rate(sample_rate);

        let wavetable = Wavetable::create_wavetable(
            // "test_wavs/CantinaBand.wav".to_string(),
            //"test_wavs/saw_440.wav".to_string(),
            "test_wavs/wavetable/chiptune_pulse.wav".to_string(),
            config.sample_rate().0,
        );
        let mut osc = Oscillator::new(0.5, 1440., wavetable);
        // osc.add_effect(Box::new(IIRLowPassFilter::new_low_pass(get_sample_rate(), 2000., 1.)));
        osc.add_effect(Box::new(ModulatedFilter::new(
            // TODO: frequency is all weird now since it gets chunked
            //       it's only calcing the LFO for the _sample time at chunk request_
            //       need to advance it into the future like we did for adsr too
            Lfo::new(LfoType::Sine, 0.5, 1.),
            Filter::StateVariableTPTFilter(StateVariableTPTFilter::new(
                get_sample_rate(),
                1000.,
                FilterType::LowPass,
            )),
            2000.,
        )));

        let wavetable2 =
            Wavetable::create_wavetable("test_wavs/wavetable/organ.wav".to_string(), config.sample_rate().0);
        let osc2 = Oscillator::new(0.2, 440., wavetable2);

        let mixer = Mixer::new(vec![osc, osc2]);

        let _ = match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), mixer, command_receiver),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), mixer, command_receiver),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), mixer, command_receiver),
        };
    });
}

fn start_gui(command_sender: crossbeam_channel::Sender<Message>) {
    //this is my tuix example from my tuix repo
    let app = Application::new(|state, window| {
        match state.add_stylesheet("bin/src/bbytheme.css") {
            Ok(_) => {}
            Err(e) => println!("Error loading stylesheet: {}", e),
        }

        window
            .set_title("basic")
            .set_background_color(state, Color::rgb(17, 21, 22))
            .set_align_items(state, AlignItems::FlexStart);

        let controller =
            Controller::new(command_sender.clone())
                .build(state, window.entity(), |builder| builder);

        // let _one = Element::new().build(state, window.entity(), |builder| {
        //     builder
        //         .class("one")
        //         .set_background_gradient(
        //             LinearGradient::new(Direction::TopToBottom)
        //                 .add_stop(GradientStop::new(
        //                     Units::Pixels(0.0),
        //                     Color::rgb(190, 90, 190),
        //                 ))
        //                 .add_stop(GradientStop::new(
        //                     Units::Pixels(30.0),
        //                     Color::rgb(50, 50, 50),
        //                 )),
        //         )
        //         .set_text("Button")
        // });
    });

    app.run();
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
    EffectsEvent(usize, EffectsEvent),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectsEvent {
    IIRFreqChange(f32),
    Enabled(bool),
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

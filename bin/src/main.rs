// TODO: write title for this thing
//! Ravetable
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

use std::path::Path;
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait};
use log::info;
use tuix::*;

use effects::filters::{Filter, FilterType, ModulatedFilter, StateVariableTPTFilter};
use effects::lfo::{Lfo, LfoType};
use effects::Effect;

use crate::gui::Controller;
use crate::mixer::{Mixer, MixerStatePacket};
use crate::playback::run;
use crate::state::{get_sample_rate, set_sample_rate};
use crate::synths::{Oscillator, Sample, Wavetable};

mod gui;
mod keyboard;
mod messages;
mod mixer;
mod playback;
mod state;
mod synths;

#[derive(Debug)]
struct Opt {
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    jack: bool,

    device: String,
}

pub type CrossbeamReceiver = crossbeam_channel::Receiver<messages::Message>;
pub type CrossbeamSender = crossbeam_channel::Sender<messages::Message>;

fn query_samples(samples_path: &str) -> Vec<Sample> {
    let base_path = Path::new(".");
    let sample_path = base_path.join(Path::new(samples_path));

    let dir = std::fs::read_dir(sample_path).unwrap();

    dir.map(|d| {
        let d = d.unwrap();
        Sample {
            name: d.file_name().to_os_string().into_string().unwrap(),
            path: d.path().into_os_string().into_string().unwrap(),
        }
    })
    .collect()
}

fn main() -> Result<(), anyhow::Error> {
    let (gui_tx, audio_rx) = crossbeam_channel::bounded(1024);
    let (_audio_tx, gui_rx) = crossbeam_channel::bounded(32);

    init_logger();

    let samples_path = "wavetable/";
    let samples = query_samples(samples_path); // and called here

    let host = cpal::default_host();

    let device = host.default_output_device().expect("Device failed");
    info!(
        "Output device: {}",
        device.name().expect("No output device found")
    );

    let config = device.default_output_config().unwrap();
    info!("Default output config: {:?}", config);
    let sample_rate = config.sample_rate();

    set_sample_rate(sample_rate);

    let wavetable = Wavetable::create_wavetable(samples[0].clone(), config.sample_rate().0);
    let mut osc = Oscillator::new(0.5, 1440., wavetable);
    osc.add_effect(Effect::ModulatedFilter(ModulatedFilter::new(
        // TODO: frequency is all weird now since it gets chunked
        //       it's only calcing the LFO for the _sample time at chunk request_
        //       need to advance it into the future like we did for adsr too
        Lfo::new(LfoType::Sine, 0.5, 1.),
        Filter::StateVariableTPTFilter(StateVariableTPTFilter::new(
            get_sample_rate(),
            2000.,
            FilterType::LowPass,
        )),
        2000.,
    )));

    let wavetable2 = Wavetable::create_wavetable(samples[0].clone(), config.sample_rate().0);
    let mut osc2 = Oscillator::new(0.2, 440., wavetable2);
    osc2.add_effect(Effect::ModulatedFilter(ModulatedFilter::new(
        Lfo::new(LfoType::Sine, 0., 1.),
        Filter::StateVariableTPTFilter(StateVariableTPTFilter::new(
            get_sample_rate(),
            2000.,
            FilterType::LowPass,
        )),
        2000.,
    )));

    let mixer = Mixer::new(vec![osc, osc2]);
    let mixer_state_packet = mixer.get_state_packet().clone();

    // Audio backend must be started first, as GUI runs on main thread because of OSX
    thread::spawn(move || {
        let _ = match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), mixer, audio_rx.clone()),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), mixer, audio_rx.clone()),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), mixer, audio_rx.clone()),
        };
    });

    start_gui(gui_tx.clone(), gui_rx.clone(), mixer_state_packet, samples);

    Ok(())
}

fn start_gui(
    tx: crossbeam_channel::Sender<messages::Message>,
    rx: crossbeam_channel::Receiver<messages::Message>,
    mixer_state_packet: MixerStatePacket,
    available_samples: Vec<Sample>,
) {
    let app = Application::new(
        |state, window| {
            match state.add_stylesheet("bin/src/bbytheme.css") {
                Ok(_) => {}
                Err(e) => panic!("Error loading stylesheet: {}", e),
            }

            window
                .set_background_color(state, Color::rgb(17, 21, 22))
                .set_justify_content(state, JustifyContent::Center)
                .set_align_items(state, AlignItems::Center);

            Controller::new(
                tx.clone(),
                rx.clone(),
                mixer_state_packet,
                available_samples,
            )
            .build(state, window.entity(), |builder| builder);
        },
        Some(830),
        Some(680),
    );

    app.run();
}

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
        .level(log::LevelFilter::Info)
        .level_for("wgpu_core", log::LevelFilter::Off)
        .level_for("gfx_backend_vulkan", log::LevelFilter::Off)
        .level_for("gfx_backend_dx11", log::LevelFilter::Off)
        .chain(std::io::stdout())
        //.chain(fern::log_file(&log_filepath).unwrap())
        .apply()
        .unwrap();
}

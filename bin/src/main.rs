// TODO: write title for this thing
//! Clipped Sine Wave assignment
//!
//! Playback code largely adapted from cpal examples: https://github.com/RustAudio/cpal/tree/master/examples
//! Output WAV is saved to "$CARGO_MANIFEST_DIR/recorded.wav".
//!
//! Katherine Philip (For CS 410P/510 Computers, Sound and Music (Spring 2021))

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::info;
use std::thread;
use tuix::Application;
use tuix::*;
use crate::utils::sample_converter::load_waveform;
use cpal::SampleFormat;

mod utils;

static THEME: &'static str = include_str!("bbytheme.css");

#[derive(Debug)]
struct Opt {
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    jack: bool,

    device: String,
}

impl Opt {
    fn from_args() -> Self {
        let app = clap::App::new("beep").arg_from_usage("[DEVICE] 'The audio device to use'");
        #[cfg(all(
            any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
            feature = "jack"
        ))]
        let app = app.arg_from_usage("-j, --jack 'Use the JACK host");
        let matches = app.get_matches();
        let device = matches.value_of("DEVICE").unwrap_or("default").to_string();

        #[cfg(all(
            any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
            feature = "jack"
        ))]
        return Opt {
            jack: matches.is_present("jack"),
            device,
        };

        #[cfg(any(
            not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
            not(feature = "jack")
        ))]
        Opt { device }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Note(f32),
    Frequency(f32),
    Amplitude(f32),
}

type CrossbeamReceiver = crossbeam_channel::Receiver<Message>;
type CrossbeamSender = crossbeam_channel::Sender<Message>;

struct OscillatorControl {
    amplitude_knob: Entity,
    frequency_knob: Entity,
    active_toggle: Entity,
}

// impl Default for OscillatorControl {
//     fn default() -> Self {
//         OscillatorControl {
//             amplitude_knob: Entity::null(),
//             frequency_knob: Entity::null(),
//             active_toggle: Entity::null(),
//         }
//     }
// }
//
// impl Widget for OscillatorControl {
//     type Ret = Entity;
//
//     fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
//         let root= HBox::new().build(state, entity, |builder| {
//             builder.set_flex_direction(FlexDirection::Column).set_padding(Units::Pixels(2.)).set_margin(Units::Pixels(4.))
//                 .set_border_width(Units::Pixels(2.)).set_border_color(Color::black())
//         });
//         let row = HBox::new().build(state, root, |builder| {
//             builder.set_justify_content(JustifyContent::SpaceEvenly).set_margin_bottom(Units::Pixels(5.0))
//         });
//         let row2 = HBox::new().build(state, root, |builder| {
//             builder.set_justify_content(JustifyContent::SpaceEvenly).set_margin_bottom(Units::Pixels(5.0))
//         });
//     }
// }

struct Controller {
    command_sender: crossbeam_channel::Sender<Message>,
    // oscillators: [OscillatorControl; 3],
    amplitude_knob: Entity,
    frequency_knob: Entity,
    active_toggle: Entity,
}

impl Controller {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        // let mut oscillators = [OscillatorControl::default(); 3];

        Controller {
            command_sender,
            amplitude_knob: Entity::null(),
            frequency_knob: Entity::null(),
            active_toggle: Entity::null(),
        }
    }
}

//TODO osc freq defaults for 2 and 3 with 440 at 1 is 523.25 and 659.25

impl Widget for Controller {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let root = HBox::new().build(state, entity, |builder| {
            builder
                .set_flex_direction(FlexDirection::Column)
                .set_padding(Units::Pixels(2.))
                .set_margin(Units::Pixels(4.))
                .set_border_width(Units::Pixels(2.))
                .set_border_color(Color::black())
        });
        let row = HBox::new().build(state, root, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_margin_bottom(Units::Pixels(5.0))
        });
        let row2 = HBox::new().build(state, root, |builder| {
            builder
                .set_justify_content(JustifyContent::SpaceEvenly)
                .set_margin_bottom(Units::Pixels(5.0))
        });

        self.amplitude_knob =
            ValueKnob::new("Amplitude", 1.0, 0.0, 1.0)
                .build(state, row, |builder| builder.set_width(Units::Pixels(50.0)));

        self.frequency_knob =
            ValueKnob::new("Frequency", 440.0, 0.0, 6000.0)
                .build(state, row, |builder| builder.set_width(Units::Pixels(50.0)));

        println!("Set focused in on_build to {}", state.focused);
        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::KeyDown(code, _) => {
                    if *code == Code::KeyZ || *code == Code::Digit5 {
                        println!("Z pressed");
                        self.command_sender.send(Message::Note(1.0)).unwrap();
                    }
                }
                WindowEvent::KeyUp(code, _) => {
                    if *code == Code::KeyZ || *code == Code::Digit5 {
                        println!("Z up");
                        self.command_sender.send(Message::Note(0.0)).unwrap();
                    }
                }
                _ => {}
            }
        }

        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                SliderEvent::ValueChanged(val) => {
                    if event.target == self.amplitude_knob {
                        self.command_sender.send(Message::Amplitude(*val)).unwrap();
                    }

                    if event.target == self.frequency_knob {
                        self.command_sender.send(Message::Frequency(*val)).unwrap();
                    }
                }

                _ => {}
            }
        }
    }
}


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
		.level(log::LevelFilter::Info)
		// Output to stdout, files, and other Dispatch configurations
		.chain(std::io::stdout())
		//.chain(fern::log_file(&log_filepath).unwrap())
		// Apply globally
		.apply()
		.unwrap();

}

fn main() -> Result<(), anyhow::Error> {

	init_logger();
    // Enumerate devices prints out all available devices and its default and supported configs
    // Run with RUST_LOG=info to see them
    let _ = utils::enumerate_devices().unwrap();


    // Creating an output writer and write a sine wave to sine.wav
    // let mut wav_writer = hound::WavWriter::create("sine.wav", spec).unwrap();
    // for t in (0..48000).map(|x| x as f32 / 48000.0) { // Gets each sample for 1 sec worth of audio
    //     let sample = (t * 440.0 * 2.0 * PI).sin(); // Generating a specific point on a sine wave that plays at 440hz
    //     let amplitude = i16::MAX as f32 * 0.25; // Scale to 1/4 max amplitude
    //     wav_writer.write_sample((sample * amplitude) as i16).unwrap(); // Actually write
    // }
    // wav_writer.finalize().unwrap();
    // println!("Successfully wrote sine.wav");

    // let mut clipped_buffer: Vec<i16> = Vec::new();
    // // Creating an output writer and write a clipped sine wave to clipped.wav + push up to a buffer
    // //	so we can play it after this
    // for t in (0..48000).map(|x| x as f32 / 48000.0) {
    //     let sample = (t * 440.0 * 2.0 * PI).sin();
    //     let amplitude = i16::MAX as f32 * 0.5;
    //
    //     let mut out_sample = sample * amplitude;
    //     out_sample = out_sample.clamp(-8192f32, 8192f32); // 8192 == i16::MAX / 4
    //
    //     clipped_buffer.push(out_sample as i16);
    // }

    let rawbuf = load_waveform("test_wavs/CantinaBand.wav",);
    play_audio(rawbuf.audio_buffer)?;
    Ok(())
}

// pub fn play_audio(buffer: Vec<f64>) -> Result<(), anyhow::Error> {
pub fn play_audio(buffer: Vec<f32>) -> Result<(), anyhow::Error> {
    let (command_sender, command_receiver) = crossbeam_channel::bounded(1024);
    thread::spawn(move || {
        // Conditionally compile with jack if the feature is specified.
        #[cfg(all(
            any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
            feature = "jack"
        ))]
        // Manually check for flags. Can be passed through cargo with -- e.g.
        // cargo run --release --example beep --features jack -- --jack
        let host = if opt.jack {
            cpal::host_from_id(cpal::available_hosts()
                .into_iter()
                .find(|id| *id == cpal::HostId::Jack)
                .expect(
                    "make sure --features jack is specified. only works on OSes where jack is available",
                )).expect("jack host unavailable")
        } else {
            cpal::default_host()
        };

        #[cfg(any(
            not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
            not(feature = "jack")
        ))]
        let host = cpal::default_host();

        // Setup the output device and stream with the default output config for outputting sound to speakers via cpal
        let device = host
            .default_output_device()
            .expect("failed to find output device");

        info!("Output device: {}", device.name()?);

        let next_value = buffer;

        // Create config packet for our default speakers, which we'll use to create an output stream later
        let config = device
            .default_output_config()
            .expect("Failed to get default input config");
        info!("Default input config: {:?}", config);

        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(
                &device,
                &config.config(),
                next_value,
                command_receiver.clone(),
            ),
            cpal::SampleFormat::I16 => run::<i16>(
                &device,
                &config.config(),
                next_value,
                command_receiver.clone(),
            ),
            cpal::SampleFormat::U16 => run::<u16>(
                &device,
                &config.config(),
                next_value,
                command_receiver.clone(),
            ),
        }
    });

    // TODO: these app shit should probably not be in a function thats called play_audio idk. move later
    let app = Application::new(|state, window| {
        state.style.parse_theme(THEME);

        Controller::new(command_sender.clone()).build(state, window.entity(), |builder| builder);
        //win_desc.with_title("Ravetable").with_inner_size(200, 200)
    });

    app.run();

    Ok(())
}

pub struct Oscillator {
    pub phi: f32,
    pub frequency: f32,
    pub amplitude: f32,
    pub enabled: bool,
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer_to_play: Vec<f32>,
    command_receiver: CrossbeamReceiver,
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channels = config.channels as usize;

    let mut buf = buffer_to_play;
    buf.reverse(); // Flip em so pop gets first samples instead of last
    let mut next_sample = move || {
        buf.pop().unwrap_or(0.)
    };

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_sample)
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::park();

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

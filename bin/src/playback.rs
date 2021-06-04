use cpal::{
    traits::{DeviceTrait, StreamTrait},
};

use crate::{synths::Oscillator, Message};
use crate::mixer::Mixer;

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut mixer: Mixer,
    command_receiver: crossbeam_channel::Receiver<Message>
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let output_channels = config.channels as usize;
    let input_channels = mixer.channels;
    // let mut samples = input_wav.samples.take().unwrap().into_iter(); //input_wav's job is basically done now

    /////// INPUT WAV VERSION
    // let mut next_value = move || {
    //     samples.next().unwrap()
    // };

    /////// SINE OSC VERSION

    let mut next_value = move || mixer.get_next_sample(&command_receiver);

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

fn write_data<T>(
    output: &mut [T],
    output_channels: usize,
    input_channels: u16,
    next_sample: &mut dyn FnMut() -> f32,
) where
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
            _ => panic!("Unsupported channels found in input audio"),
        }
    }
}

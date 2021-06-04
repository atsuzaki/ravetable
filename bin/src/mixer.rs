use crate::{synths::Oscillator, Message};
use log::{error, warn};

pub struct Mixer {
    oscillators: Vec<Oscillator>,
    pub channels: u16,
    samples_since_last_gui_poll: u32,
}

impl Mixer {
    pub fn new<T: Into<Vec<Oscillator>>>(oscillators: T) -> Mixer {
        Mixer {
            oscillators: oscillators.into(),
            channels: 2,
            samples_since_last_gui_poll: 0,
        }
    }

    pub fn get_next_sample(
        &mut self,
        command_receiver: &crossbeam_channel::Receiver<Message>,
    ) -> f32 {
        // Poll crossbeam channel for msg
        self.samples_since_last_gui_poll += 1;
        if self.samples_since_last_gui_poll > 250 {
            // This should be user configurable at some point
            self.samples_since_last_gui_poll = 0;

            match command_receiver.try_recv() {
                // TODO
                Ok(val) => match val {
                    Message::Note(_) => {
                        warn!("Note is unimplemented");
                    }
                    Message::Frequency(_) => {
                        warn!("Frequency is unimplemented");
                    }
                    Message::Amplitude(gain) => {
                        self.oscillators[0].set_gain(gain);
                    }
                },
                Err(_) => {} // This happens constantly and only means there was nothing to receive
            }
        }

        // Add up all the get_next_sample()s from the oscillators, divide by # of osc
        let output_channels = self.channels;
        let unclamped = self
            .oscillators
            .iter_mut()
            .fold(0., |accum, o| match output_channels {
                // All samples are stereo-fied and here we work under such assumptions
                1 => accum + ((o.get_next_sample() + o.get_next_sample()) / 2.),
                _ => accum + o.get_next_sample(),
            })
            / (self.oscillators.len() as f32);

        // TODO: make better limiter
        unclamped.clamp(-1.0f32, 1.0f32)
    }
}

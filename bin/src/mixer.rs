use crate::synths::Oscillator;

pub struct Mixer {
    oscillators: Vec<Oscillator>,
    pub channels: u16,
}

impl Mixer {
    pub fn new<T: Into<Vec<Oscillator>>>(oscillators: T) -> Mixer {
        Mixer {
            oscillators: oscillators.into(),
            channels: 2,
        }
    }

    pub fn get_next_sample(&mut self) -> f32 {
        // Add up all the get_next_sample()s from the oscillators, divide by # of osc
		let output_channels= self.channels;
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
        unclamped.clamp( -1.0f32, 1.0f32)
    }
}

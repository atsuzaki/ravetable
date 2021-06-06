use crate::{synths::Oscillator, Message, EffectsEvent, get_sample_rate};
use log::{error, warn};
use itertools::Itertools;
use effects::filters::IIRLowPassFilter;

pub struct Mixer {
    pub channels: u16,
    pub chunk_size: u32,

    chunk_buffer: Vec<f32>,
    chunk_buffer_index: usize,

    oscillators: Vec<Oscillator>,
    samples_since_last_gui_poll: u32,
}

impl Mixer {
    pub fn new<T: Into<Vec<Oscillator>>>(oscillators: T) -> Mixer {
        let chunk_size = 128;

        Mixer {
            oscillators: oscillators.into(),
            channels: 2,
            chunk_size: chunk_size,
            chunk_buffer_index: chunk_size as usize,
            chunk_buffer: vec![],
            samples_since_last_gui_poll: 0,
        }
    }

    fn poll_crossbeam_channel(&mut self,
                              command_receiver: &crossbeam_channel::Receiver<Message>,
    ) {
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
                    Message::Frequency(_) => { }
                    Message::Amplitude(gain) => {
                        self.oscillators[0].set_gain(gain);
                    }
                    Message::EffectsEvent(idx, event) => {
                        match event {
                            EffectsEvent::IIRFreqChange(f) => {
                                // effects[idx[ is a trait object, need to cast it back to what it was or have a generic thing to call to handle events
                                let fx = &mut self.oscillators[0].effects[idx];
                                let mut fx = fx.as_any_mut().downcast_mut::<IIRLowPassFilter>().expect("Downcast failed");
	                            fx.set_frequency(get_sample_rate(), f, 1.);

                            }
                            EffectsEvent::Enabled(_) => {}
                        }
                    }
                },
                Err(_) => {} // This happens constantly and only means there was nothing to receive
            }
        }
    }

    fn get_next_chunk(
        &mut self,
    ) {
        // Add up all the get_next_sample()s from the oscillators, divide by # of osc
        let output_channels = self.channels;
        let chunk_size = self.chunk_size;

        let sample_count = chunk_size;
        let mut chunk_summed: Vec<f32> = Vec::new();

        let chunks = self.oscillators.iter_mut().map(|o| {
            let mut chunks = o.get_next_chunk(sample_count);

            for e in &mut o.effects {
                e.process_samples(&mut chunks);
            }

            chunks
        }).collect_vec();

        for i in 0..sample_count as usize {
            let combined_sample = chunks.iter().map(|chunk| chunk[i]).sum::<f32>() / chunks.len() as f32;
            chunk_summed.push(combined_sample);
        }

        self.chunk_buffer = chunk_summed;
    }

    pub fn get_next_sample_chunked(
        &mut self,
        command_receiver: &crossbeam_channel::Receiver<Message>,
    ) -> f32 {
        self.poll_crossbeam_channel(command_receiver);
        if self.chunk_buffer_index >= self.chunk_size as usize {
            self.chunk_buffer_index = 0;
            self.get_next_chunk();
        }

        let curr_index = self.chunk_buffer_index;
        self.chunk_buffer_index += 1;

        self.chunk_buffer[curr_index]
    }

	pub fn get_next_sample(
		&mut self,
		command_receiver: &crossbeam_channel::Receiver<Message>,
	) -> f32 {
		self.poll_crossbeam_channel(command_receiver);

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

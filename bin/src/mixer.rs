use itertools::Itertools;

use crate::messages::{
    EnvelopeParams, LfoParams, Message, ModulatedFilterParams, OscParams, StateVarTPTFilterParams,
};
use crate::state::{advance_sample_clock, get_sample_clock};
use crate::synths::OscStatePacket;
use crate::synths::Oscillator;
use effects::filters::Filter;
use effects::{get_sample_rate, Effect};

#[derive(Clone)]
pub struct MixerStatePacket {
    pub oscillators: Vec<OscStatePacket>,
}

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
            chunk_size,
            chunk_buffer_index: chunk_size as usize,
            chunk_buffer: vec![],
            samples_since_last_gui_poll: 0,
        }
    }

    fn poll_crossbeam_channel(&mut self, command_receiver: &crossbeam_channel::Receiver<Message>) {
        // Poll crossbeam channel for msg
        self.samples_since_last_gui_poll += 1;
        if self.samples_since_last_gui_poll > 250 {
            // This should be user configurable at some point
            self.samples_since_last_gui_poll = 0;

            match command_receiver.try_recv() {
                Ok(val) => match val {
                    Message::Note(note) => {
                        let clock = get_sample_clock();
                        if note >= 1.0 {
                            self.oscillators.iter_mut().for_each(|o| o.trigger(clock));
                        } else if note == 0.0 {
                            self.oscillators.iter_mut().for_each(|o| o.release(clock));
                        }
                    }
                    Message::Frequency(frq) => {
                        self.oscillators
                            .iter_mut()
                            .for_each(|o| o.set_frequency(frq));
                    }
                    Message::EnvelopeChange(id, param) => {
                        let mut osc = &mut self.oscillators[id];
                        match param {
                            EnvelopeParams::Delay(val) => osc.envelope.adsr_values.delay = val,
                            EnvelopeParams::Attack(val) => osc.envelope.adsr_values.attack = val,
                            EnvelopeParams::Decay(val) => osc.envelope.adsr_values.decay = val,
                            EnvelopeParams::Sustain(val) => osc.envelope.adsr_values.sustain = val,
                            EnvelopeParams::Release(val) => osc.envelope.adsr_values.release = val,
                        }
                    }
                    Message::OscChange(id, param) => {
                        let osc = &mut self.oscillators[id];
                        match param {
                            OscParams::Gain(gain) => osc.set_gain(gain),
                            OscParams::SampleChange(sample) => {
                                osc.queue_change_wavetable(sample);
                            }
                        }
                    }
                    Message::ModulatedFilterParams(id, effect_id, param) => {
                        // TODO: fix this gnarly match
                        if let Effect::ModulatedFilter(e) =
                            &mut self.oscillators[id].effects[effect_id]
                        {
                            match &param {
                                ModulatedFilterParams::Filter(f) => {
                                    if let Filter::StateVariableTPTFilter(filter) = &mut e.filter {
                                        match f {
                                            StateVarTPTFilterParams::FilterType(v) => {
                                                filter.set_filter_type(*v);
                                            }
                                            StateVarTPTFilterParams::Frequency(v) => {
                                                filter.set_frequency(get_sample_rate(), *v);
                                            }
                                            StateVarTPTFilterParams::Resonance(v) => {
                                                filter.set_resonance(get_sample_rate(), *v);
                                            }
                                        };
                                    };
                                }
                                ModulatedFilterParams::Lfo(f) => {
                                    let lfo = &mut e.lfo;
                                    // TODO
                                    match f {
                                        LfoParams::LfoType(v) => {
                                            lfo.set_waveform(*v);
                                        }
                                        LfoParams::Frequency(v) => {
                                            lfo.set_frequency(*v);
                                        }
                                        LfoParams::Phase(v) => {
                                            lfo.set_phase(*v);
                                        }
                                    }
                                }
                                ModulatedFilterParams::BaseFrequency(frq) => e.set_frequency(*frq),
                            }
                        }
                    }
                },
                Err(_) => {} // This happens constantly and only means there was nothing to receive
            }
        }
    }

    fn get_next_chunk(&mut self) {
        // Add up all the get_next_sample()s from the oscillators, divide by # of osc
        let chunk_size = self.chunk_size;

        let sample_count = chunk_size;
        let mut chunk_summed: Vec<f32> = Vec::new();

        let frame_sample_clock = get_sample_clock();

        let chunks = self
            .oscillators
            .iter_mut()
            .map(|o| {
                let mut chunks = o.get_next_chunk(sample_count, frame_sample_clock);

                for e in &mut o.effects {
                    match e {
                        Effect::ModulatedFilter(e) => {
                            e.process_samples(frame_sample_clock, &mut chunks)
                        }
                        Effect::IIRFilter(e) => e.process_samples(frame_sample_clock, &mut chunks),
                        Effect::StateVariablePTPFilter(e) => {
                            e.process_samples(frame_sample_clock, &mut chunks)
                        }
                    }
                }

                chunks
            })
            .collect_vec();

        for i in 0..sample_count as usize {
            let combined_sample =
                chunks.iter().map(|chunk| chunk[i]).sum::<f32>() / chunks.len() as f32;
            chunk_summed.push(combined_sample);
        }

        advance_sample_clock(chunk_size as u64);

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

    pub fn get_state_packet(&self) -> MixerStatePacket {
        MixerStatePacket {
            oscillators: self
                .oscillators
                .iter()
                .map(|o| o.get_state_packet())
                .collect(),
        }
    }

    // pub fn get_next_sample(
    // 	&mut self,
    // 	command_receiver: &crossbeam_channel::Receiver<Message>,
    // ) -> f32 {
    // 	self.poll_crossbeam_channel(command_receiver);
    //
    // 	// Add up all the get_next_sample()s from the oscillators, divide by # of osc
    // 	let output_channels = self.channels;
    // 	let unclamped = self
    // 		.oscillators
    // 		.iter_mut()
    // 		.fold(0., |accum, o| match output_channels {
    // 			// All samples are stereo-fied and here we work under such assumptions
    // 			1 => accum + ((o.get_next_sample() + o.get_next_sample()) / 2.),
    // 			_ => accum + o.get_next_sample(),
    // 		})
    // 		/ (self.oscillators.len() as f32);
    //
    // 	// TODO: make better limiter
    // 	unclamped.clamp(-1.0f32, 1.0f32)
    // }
}

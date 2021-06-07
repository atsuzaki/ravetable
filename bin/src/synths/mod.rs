use hound::WavSpec;
use itertools::Itertools;
use samplerate::{ConverterType, Samplerate};
use std::ptr;

use effects::adsr::{ADSREnvelope, ADSR};
use effects::filters::IIRLowPassFilter;
use effects::Effect;
use crate::state::get_sample_rate;

pub struct Wavetable {
    pub sample_table: Vec<f32>, // Buffer of samples from .wav file
    pub spec: WavSpec,
    pub file_path: String,
}

impl Wavetable {
    pub fn create_wavetable(filename: String, output_sample_rate: u32) -> Wavetable {
        let reader = hound::WavReader::open(&filename).unwrap();
        let mut input_wav_spec = reader.spec();

        let samples = reader.into_samples::<f32>();
        let mut fsamples: Vec<f32> = samples.map(|f| f.unwrap()).collect();

        //Double the mono channel to stereo via interleaving manually
        if input_wav_spec.channels == 1 {
            input_wav_spec.channels = 2;

            let sample_iter1 = fsamples.iter();
            let sample_iter2 = fsamples.iter();
            fsamples = sample_iter1
                .interleave(sample_iter2)
                .map(|&x| x)
                .collect::<Vec<f32>>();
        }

        if input_wav_spec.sample_rate != output_sample_rate {
            println!("Converting sample");
            // Instanciate a new converter.
            let sample_rate_converter = Samplerate::new(
                ConverterType::SincBestQuality,
                input_wav_spec.sample_rate,
                output_sample_rate,
                input_wav_spec.channels as usize,
            )
            .unwrap();

            // Resample the input from input sample rate to output sample rate
            fsamples = sample_rate_converter.process_last(&fsamples).unwrap();
        }

        Wavetable {
            sample_table: fsamples,
            spec: input_wav_spec,
            file_path: filename,
        }
    }

    #[inline(always)]
    pub fn get_num_samples(&self) -> usize {
        self.sample_table.len()
    }
}

const CLAMP_COEFF: f32 = 3.;

pub struct Oscillator {
    gain: f32,
    frequency: f32,
    current_index: f32,
    table_delta: f32,
    table_size_index: usize,

    pub wavetable: Wavetable,
    pub effects: Vec<Box<dyn Effect + Send>>,
    envelope: ADSREnvelope,
}

impl Oscillator {
    pub fn new(gain: f32, frequency: f32, wavetable: Wavetable) -> Oscillator {
        let mut osc = Oscillator {
            gain,
            frequency,
            table_size_index: &wavetable.get_num_samples() - 1,
            wavetable,

            current_index: 0.,
            table_delta: 0.,
            effects: vec![],
            envelope: ADSREnvelope::new(ADSR::default()),
        };

        osc.add_effect(Box::new(IIRLowPassFilter::new_low_pass(
            get_sample_rate(),
            frequency * CLAMP_COEFF,
            1.,
        )));
        osc.update_table_delta();
        osc
    }

    pub fn add_effect(&mut self, effect: Box<dyn Effect + Send>) {
        self.effects.push(effect);
    }

    pub fn reset(&mut self) {
        self.current_index = 0.;
        self.envelope.reset();
    }

    pub fn trigger(&mut self, sample_clock: u64) {
        self.envelope.trigger(sample_clock);
    }

    pub fn release(&mut self, sample_clock: u64) {
        self.envelope.release(sample_clock);
    }

    #[inline(always)]
    pub fn get_next_sample(&mut self, sample_time: u64) -> f32 {
        //println!("{}", sample_time);
        let index0 = self.current_index as usize;
        let index1 = if index0 == self.table_size_index {
            0
        } else {
            index0 + 1
        };

        let frac = self.current_index - (index0 as f32);

        let value0 = self.wavetable.sample_table[index0];
        let value1 = self.wavetable.sample_table[index1];

        let current_sample = value0 + frac * (value1 - value0);

        self.current_index += self.table_delta; // todo: keep everything as is, but modify table_delta based on last_pos shit?
        let new_index = self.current_index;

        if new_index > self.table_size_index as f32 {
            self.current_index -= self.table_size_index as f32;
        }

	    let adsr_sample = self.envelope.get_next_sample(sample_time);

        current_sample * self.gain * adsr_sample
    }

    pub fn get_next_chunk(&mut self, chunk_size: u32, sample_clock_start: u64) -> Vec<f32> {
        //let sample_clock_start = get_sample_clock();
        let mut result = Vec::with_capacity(chunk_size as usize);
        for i in 0..chunk_size {
            result.push(self.get_next_sample(sample_clock_start + i as u64));
        }
        result
    }

    pub fn get_channels(&self) -> u16 {
        self.wavetable.spec.channels
    }

    // Not sure if this is exactly the correct way but this seems to produce what'm after
    // have an assumed frq: 440
    // take diff of assumed frq and asked frq, calc the diff and translate it to table_delta
    fn update_table_delta(&mut self) {
        let assumed_frq = 440.; // TODO: const this
        let frq_fraq = self.frequency / assumed_frq;
        // Assuming that at assumed_frq it needs to move at 1 index per, calc how much table_delta
        // is needed to play at the new frequency
        self.table_delta = frq_fraq; // ok, changing the table delta will change the speed too.
    }

    fn update_low_pass_filter(&mut self) {
        // Internal low pass filter we use for clamping is always index 0, since we always add it first
        self.effects[0]
            .as_any_mut()
            .downcast_mut::<IIRLowPassFilter>()
            .unwrap()
            .set_frequency(get_sample_rate(), self.frequency * CLAMP_COEFF, 1.);
    }

    pub fn set_gain(&mut self, new_gain: f32) {
        self.gain = new_gain;
    }

    pub fn set_frequency(&mut self, new_frequency: f32) {
        self.frequency = new_frequency;
        self.update_low_pass_filter();
        self.update_table_delta();
    }
}

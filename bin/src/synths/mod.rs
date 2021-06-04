use samplerate::{ConverterType, Samplerate};
use hound::WavSpec;
use std::ptr;
use itertools::Itertools;

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

pub struct Oscillator {
    gain: f32,
    current_index: f32,
    table_delta: f32,

    pub wavetable: Wavetable,
    table_size_index: usize,
}

impl Oscillator {
    pub fn new(gain: f32, wavetable: Wavetable) -> Oscillator {
        let mut osc = Oscillator {
            gain,
            table_size_index: &wavetable.get_num_samples() - 1,
            wavetable,

            current_index: 0.,
            table_delta: 0.,
        };

	    osc.update_table_delta();
        osc
    }

	#[inline(always)]
    pub fn get_next_sample(&mut self) -> f32 {
        let index0 = self.current_index as usize;
        let index1 = if index0 == self.table_size_index  {
            0
        } else {
            index0 + 1
        };

        let frac = self.current_index - (index0 as f32);

        let value0 = self.wavetable.sample_table[index0];
        let value1 = self.wavetable.sample_table[index1];

        let current_sample = value0 + frac * (value1 - value0);

        self.current_index += self.table_delta;
        let new_index = self.current_index;

        if new_index > self.table_size_index as f32 {
            self.current_index -= self.table_size_index as f32;
        }

		//println!("Next sample is {}", &current_sample);
		//println!("{} - {} - {} ", current_sample, self.gain, get_sample_rate());

        return current_sample * self.gain;
    }

    pub fn get_channels(&self) -> u16 {
        self.wavetable.spec.channels
    }

	fn update_table_delta(&mut self) {
		//Currently hard set to 1 as we don't want any timing to change yet
		self.table_delta = 1.0;
	}

    pub fn set_gain(&mut self, new_gain: f32) {
        self.gain = new_gain;
    }
}

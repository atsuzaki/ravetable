//! IIR Filter, mostly adapted from oxcable and JUCE

use std::f32::consts::PI;
use crate::Effect;
use std::any::Any;

// TODO: future work, use a filter that's more friendly to modulation
pub struct IIRLowPassFilter {
    v1: f32,
    v2: f32,

    c0: f32,
    c1: f32,
    c2: f32,
    c3: f32,
    c4: f32,
}

impl IIRLowPassFilter {
    pub fn new(c1: f32, c2: f32, c3: f32, c4: f32, c5: f32, c6: f32) -> IIRLowPassFilter {
        let a = 1.0 / c4;

        IIRLowPassFilter {
            v1: 0.,
            v2: 0.,

            c0: (c1 * a),
            c1: (c2 * a),
            c2: (c3 * a),
            c3: (c5 * a),
            c4: (c6 * a),
        }
    }

	pub fn set_frequency(&mut self, sample_rate: f32, frequency: f32, q: f32) {
		let n = 1. / (PI * frequency / sample_rate).tan();
		let n_squared = n * n;
		let c1_base = 1. / (1. + 1. / q * n + n_squared);

		let c1 = c1_base;
		let c2 = c1 * 2.0;
		let c3 = c1;
		let c4 = 1.;
		let c5 = c1 * 2. * (1. - n_squared);
		let c6 = c1 * (1. - 1. / q * n + n_squared);

		let a = 1.0 / c4; // TODO: this is pointless for low pass bc c4 is hard coded to 1.

		self.v1 = 0.;
		self.v2 = 0.;
		self.c0 = (c1 * a);
		self.c1 = (c2 * a);
		self.c2 = (c3 * a);
		self.c3 = (c5 * a);
		self.c4 = (c6 * a);
	}

    pub fn new_low_pass(sample_rate: f32, frequency: f32, q: f32) -> IIRLowPassFilter {
        assert!(sample_rate > 0.);
        assert!(frequency > 0. && frequency <= sample_rate * 0.5);
        assert!(q > 0.);

        let n = 1. / (PI * frequency / sample_rate).tan();
        let n_squared = n * n;
        let c1 = 1. / (1. + 1. / q * n + n_squared);

        IIRLowPassFilter::new(
            c1,
            c1 * 2.0,
            c1,
            1.,
            c1 * 2. * (1. - n_squared),
            c1 * (1. - 1. / q * n + n_squared),
        )
    }
}

impl Effect for IIRLowPassFilter {
    fn process_samples(&mut self, samples: &mut [f32]) {
        let IIRLowPassFilter {
            v1,
            v2,
            c0,
            c1,
            c2,
            c3,
            c4,
        } = *self;

        let mut lv1 = v1;
        let mut lv2 = v2;

        for i in 0..samples.len() {
            let sample = samples[i];

            let out = c0 * sample + lv1;
            samples[i] = out as f32;

            lv1 = c1 * sample - c3 * out + lv2;
            lv2 = c2 * sample - c4 * out;
        }

        self.v1 = lv1;
        self.v2 = lv2;
    }

	fn as_any(&self) -> &dyn Any {
		self
	}

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

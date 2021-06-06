//! IIR Filter, mostly adapted from oxcable and JUCE

use std::f32::consts::PI;
use crate::Effect;

pub struct IIRFilter {
    v1: f32,
    v2: f32,

    c0: f32,
    c1: f32,
    c2: f32,
    c3: f32,
    c4: f32,
}

impl IIRFilter {
    pub fn new(c1: f32, c2: f32, c3: f32, c4: f32, c5: f32, c6: f32) -> IIRFilter {
        let a = 1.0 / c4;

        IIRFilter {
            v1: 0.,
            v2: 0.,

            c0: (c1 * a),
            c1: (c2 * a),
            c2: (c3 * a),
            c3: (c5 * a),
            c4: (c6 * a),
        }
    }

    pub fn new_low_pass(sample_rate: f32, frequency: f32, q: f32) -> IIRFilter {
        assert!(sample_rate > 0.);
        assert!(frequency > 0. && frequency <= sample_rate * 0.5);
        assert!(q > 0.);

        let n = 1. / (PI * frequency / sample_rate).tan();
        let n_squared = n * n;
        let c1 = 1. / (1. + 1. / q * n + n_squared);

        IIRFilter::new(
            c1,
            c1 * 2.0,
            c1,
            1.,
            c1 * 2. * (1. - n_squared),
            c1 * (1. - 1. / q * n + n_squared),
        )
    }
}

impl Effect for IIRFilter {
    fn process_samples(&mut self, samples: &mut [f32]) {
        let IIRFilter {
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
}

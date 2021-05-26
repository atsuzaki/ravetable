//! IIR Filter, mostly adapted from oxcable and JUCE

use std::f64::consts::PI;

pub struct IIRFilter {
    v1: f64,
    v2: f64,

    c0: f64,
    c1: f64,
    c2: f64,
    c3: f64,
    c4: f64,
}

impl IIRFilter {
    pub fn new(c1: f64, c2: f64, c3: f64, c4: f64, c5: f64, c6: f64) -> IIRFilter {
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

    pub fn new_low_pass(sample_rate: f64, frequency: f64, q: f64) -> IIRFilter {
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

    pub fn process_samples(&mut self, samples: &[f64], output: &mut [f64]) {
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

        for (i, sample) in samples.iter().enumerate() {
            let sample = *sample;

            let out = c0 * sample + lv1;
            output[i] = out as f64;

            lv1 = c1 * sample - c3 * out + lv2;
            lv2 = c2 * sample - c4 * out;
        }

        self.v1 = lv1;
        self.v2 = lv2;
    }
}

//! Filters, mostly adapted from oxcable and JUCE
use crate::{get_sample_rate, Effect};
use std::any::Any;
use std::f32::consts::PI;

use crate::lfo::Lfo;
use num_traits::FloatConst;

pub struct ModulatedFilter {
    lfo: Lfo,
    filter: Filter,
    base_frequency: f32,
}

impl ModulatedFilter {
    pub fn new(lfo: Lfo, filter: Filter, base_frequency: f32) -> ModulatedFilter {
        ModulatedFilter {
            lfo,
            filter,
            base_frequency,
        }
    }
}

impl Effect for ModulatedFilter {
    fn process_samples(&mut self, sample_clock: u64, samples: &mut [f32]) {
        let freq = self.base_frequency * self.lfo.get_sample(sample_clock, 1.0, 20_000.0);
        match &mut self.filter {
            Filter::IIRLowPassFilter(filter) => {
                filter.set_frequency(get_sample_rate(), freq);
                filter.process_samples(sample_clock, samples);
            }
            Filter::StateVariableTPTFilter(filter) => {
                filter.set_frequency(get_sample_rate(), freq);
                filter.process_samples(sample_clock, samples);
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub enum Filter {
    IIRLowPassFilter(IIRLowPassFilter),
    StateVariableTPTFilter(StateVariableTPTFilter),
}

pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

pub struct StateVariableTPTFilter {
    g: f32,
    h: f32,
    r2: f32,
    s1: Vec<f32>,
    s2: Vec<f32>,

    filter_type: FilterType,
    cutoff_frequency: f32,
    resonance: f32,

    channels: u16,
}

impl StateVariableTPTFilter {
    pub fn new(
        sample_rate: f32,
        cutoff_frequency: f32,
        filter_type: FilterType,
    ) -> StateVariableTPTFilter {
        let mut s = StateVariableTPTFilter {
            g: 0.0,
            h: 0.0,
            r2: 0.0,
            s1: vec![2., 2.], // TODO: these are supposed be as big as # of channels (?)
            s2: vec![2., 2.],
            filter_type,
            cutoff_frequency,
            resonance: 1.0 / f32::sqrt(2.0),
            channels: 1,
        };
        s.set_frequency(sample_rate, cutoff_frequency);
        s
    }

    pub fn set_frequency(&mut self, sample_rate: f32, new_frequency: f32) {
        self.cutoff_frequency = new_frequency;

        self.g = (f32::PI() * self.cutoff_frequency / sample_rate).tan();
        self.r2 = 1.0 / self.resonance;
        self.h = 1.0 / (1.0 + self.r2 * self.g + self.g * self.g);
    }
}

impl Effect for StateVariableTPTFilter {
    fn process_samples(&mut self, _samples_clock: u64, samples: &mut [f32]) {
        for i in 0..samples.len() {
            let Self {
                g, r2, channels, h, ..
            } = *self;
            let channels = channels as usize - 1;

            let ls1 = self.s1[channels];
            let ls2 = self.s1[channels];

            let yhp = h * (samples[i] - ls1 * (g * r2) - ls2);

            let ybp = yhp * g + ls1;
            self.s1[channels] = yhp * g + ybp;

            let ylp = ybp * g + ls2;
            self.s2[channels] = ybp * g + ylp;

            samples[i] = match self.filter_type {
                FilterType::LowPass => ylp,
                FilterType::HighPass => ybp,
                FilterType::BandPass => yhp,
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

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

    pub fn set_frequency(&mut self, sample_rate: f32, frequency: f32) {
        let q = 1.0;
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
        self.c0 = c1 * a;
        self.c1 = c2 * a;
        self.c2 = c3 * a;
        self.c3 = c5 * a;
        self.c4 = c6 * a;
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
    fn process_samples(&mut self, _samples_clock: u64, samples: &mut [f32]) {
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

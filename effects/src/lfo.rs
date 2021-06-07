use crate::get_sample_rate;
use num_traits::FloatConst;

#[derive(Clone, Copy, Debug)]
pub enum LfoType {
    Sine,
    Saw,
    Square,
}

pub struct Lfo {
    pub waveform: LfoType,
    pub frequency: f32,
    pub phase: f32,

    pos: f32,
    last_update: u64,
}

impl Lfo {
    pub fn new(waveform: LfoType, frequency: f32, phase: f32) -> Lfo {
        Lfo {
            waveform,
            frequency,
            phase,
            pos: 0.0,
            last_update: 0,
        }
    }

    pub fn get_sample(&mut self, sample_clock: u64, min_value: f32, max_value: f32) -> f32 {
        let dt = (sample_clock - self.last_update) as f32;
        let speed = self.frequency / get_sample_rate();
        self.pos += speed * dt;
        self.last_update += dt as u64;

        if self.pos > 1. {
            self.pos -= 1.;
        }

        let unscaled = match self.waveform {
            LfoType::Sine => (self.pos * f32::TAU()).sin(),
            LfoType::Saw => (self.pos * 2.) - 1.,
            LfoType::Square => {
                if self.pos < self.phase {
                    1.
                } else {
                    -1.
                }
            }
        };

        let old_range = 2.; //Because it's -1 to 1
        let new_range = max_value - min_value;
        let old_value = unscaled;
        let old_min = -1.;
        let scaled = (((old_value - old_min) * new_range) / old_range) + min_value;
        scaled / max_value
    }
}

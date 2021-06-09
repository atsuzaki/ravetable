//! ADSR envelope adapted from Yazz: https://github.com/icsga/Yazz
//!

use crate::get_sample_rate;

#[derive(Clone, Copy)]
pub struct ADSR {
    pub delay: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for ADSR {
    fn default() -> Self {
        ADSR {
            delay: 0.0,
            attack: 10000.,
            decay: 5000.,
            sustain: 100000.0,
            release: 100000.,
        }
    }
}

impl ADSR {
    pub fn convert_adsr_from_sample_clock(adsr: &ADSR) -> ADSR {
        let sample_rate = get_sample_rate();
        ADSR {
            delay: adsr.delay / sample_rate,
            attack: adsr.attack / sample_rate,
            decay: adsr.decay / sample_rate,
            sustain: adsr.sustain / sample_rate,
            release: adsr.release / sample_rate,
        }
    }

    pub fn convert_value_from_time(value: f32) -> f32 {
        let sample_rate = get_sample_rate();
        value * sample_rate
    }

    pub fn convert_adsr_from_time(adsr: &ADSR) -> ADSR {
        let sample_rate = get_sample_rate();
        ADSR {
            delay: adsr.delay * sample_rate,
            attack: adsr.attack * sample_rate,
            decay: adsr.decay * sample_rate,
            sustain: adsr.sustain * sample_rate,
            release: adsr.release * sample_rate,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ADSREnvelopeState {
    Idle,
    Delay,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct ADSREnvelope {
    pub adsr_values: ADSR,
    state: ADSREnvelopeState,

    start_time: u64,
    end_time: u64,

    increment: f32,
    last_value: f32,
}

impl ADSREnvelope {
    pub fn new(adsr_values: ADSR) -> ADSREnvelope {
        ADSREnvelope {
            adsr_values,
            state: ADSREnvelopeState::Idle,
            start_time: 0,
            end_time: 0,
            increment: 0.0,
            last_value: 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != ADSREnvelopeState::Idle
    }

    pub fn reset(&mut self) {
        self.end_time = 0;
        // self.state = ADSREnvelopeState::Idle;
    }

    pub fn trigger(&mut self, sample_clock: u64) {
        let sample_time = sample_clock - self.start_time;
        let state = if self.adsr_values.delay > 0. {
            ADSREnvelopeState::Delay
        } else {
            ADSREnvelopeState::Attack
        };

        self.change_state(state, sample_time);
    }

    pub fn release(&mut self, sample_clock: u64) {
        let sample_time = sample_clock - self.start_time;
        if self.state != ADSREnvelopeState::Release {
            self.change_state(ADSREnvelopeState::Release, sample_time);
        }
    }

    pub fn get_next_sample(&mut self, sample_clock: u64) -> f32 {
        let sample_time = sample_clock - self.start_time;

        match self.state {
            ADSREnvelopeState::Idle => return 0.,
            ADSREnvelopeState::Delay => {
                if sample_time >= self.end_time {
                    self.change_state(ADSREnvelopeState::Attack, sample_time);
                }
            }
            ADSREnvelopeState::Attack => {
                self.last_value += self.increment;
                if sample_time >= self.end_time {
                    self.change_state(ADSREnvelopeState::Decay, sample_time);
                }
            }
            ADSREnvelopeState::Decay => {
                self.last_value += self.increment;
                if sample_time >= self.end_time {
                    self.change_state(ADSREnvelopeState::Sustain, sample_time);
                }
            }
            ADSREnvelopeState::Sustain => {
                self.last_value = self.adsr_values.sustain;
            }
            ADSREnvelopeState::Release => {
                self.last_value += self.increment;
                if sample_time >= self.end_time {
                    self.change_state(ADSREnvelopeState::Idle, sample_time);
                }
            }
        };

        self.last_value = self.last_value.clamp(0., 1.);
        self.last_value
    }

    fn change_state(&mut self, new_state: ADSREnvelopeState, sample_time: u64) {
        self.state = new_state;
        match new_state {
            ADSREnvelopeState::Idle => self.last_value = 0.,
            ADSREnvelopeState::Delay => {
                self.last_value = 0.;
                self.end_time = sample_time + self.adsr_values.delay as u64;
            }
            ADSREnvelopeState::Attack => {
                self.set_increment(0., 1., self.adsr_values.attack);

                // Accounting for if we didn't start at zero
                let frac = 1.0 - self.last_value;
                self.set_end_time(sample_time, self.adsr_values.attack * frac);
            }
            ADSREnvelopeState::Decay => {
                self.set_increment(1., self.adsr_values.sustain, self.adsr_values.decay);
                self.set_end_time(sample_time, self.adsr_values.decay);
            }
            ADSREnvelopeState::Sustain => {}
            ADSREnvelopeState::Release => {
                self.set_increment(self.last_value, 0., self.adsr_values.release);
                self.set_end_time(sample_time, self.adsr_values.release);
            }
        }
    }

    fn set_increment(&mut self, from: f32, to: f32, dur: f32) {
        self.increment = (to - from) / dur;
    }

    fn set_end_time(&mut self, sample_time: u64, end_time: f32) {
        self.end_time = sample_time + end_time as u64;
    }
}

use crate::synths::Sample;
use effects::filters::FilterType;
use effects::lfo::LfoType;

#[derive(Clone, Debug, PartialEq)]
pub enum OscParams {
    Gain(f32),
    SampleChange(Sample),
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnvelopeParams {
    Delay(f32),
    Attack(f32),
    Decay(f32),
    Sustain(f32),
    Release(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LfoParams {
    LfoType(LfoType),
    Frequency(f32),
    Phase(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum StateVarTPTFilterParams {
    FilterType(FilterType),
    Frequency(f32),
    Resonance(f32),
}

// Wish I had typescript union/intersections for defining these
#[derive(Clone, Debug, PartialEq)]
pub enum ModulatedFilterParams {
    BaseFrequency(f32),
    Filter(StateVarTPTFilterParams),
    Lfo(LfoParams),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Note(f32),
    Frequency(f32),

    OscChange(usize, OscParams),
    EnvelopeChange(usize, EnvelopeParams),

    // osc_id, filter_id, param
    ModulatedFilterParams(usize, usize, ModulatedFilterParams),
}

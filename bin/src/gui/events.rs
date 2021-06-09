use crate::synths::Sample;

// id, value
#[derive(PartialEq, Clone, Debug)]
pub enum OscillatorControlEvent {
    GainChange(usize, f32),
    FreqChange(usize, f32),
    OscWavetableChange(usize, usize)
}

#[derive(PartialEq, Clone, Debug)]
pub enum EnvelopeControlEvent {
    DelayChange(usize, f32),
    AttackChange(usize, f32),
    DecayChange(usize, f32),
    SustainChange(usize, f32),
    ReleaseChange(usize, f32)
}


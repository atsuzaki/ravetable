use crate::{EnvelopeParams, OscParams};

// id, value
#[derive(PartialEq, Clone, Debug)]
pub enum SynthControlEvent {
    OscillatorControl(usize, OscParams),
    Envelope(usize, EnvelopeParams),
}

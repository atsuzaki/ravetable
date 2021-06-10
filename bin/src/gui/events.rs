use crate::messages::{EnvelopeParams, ModulatedFilterParams, OscParams};

// id, value
#[derive(PartialEq, Clone, Debug)]
pub enum SynthControlEvent {
    OscillatorControl(usize, OscParams),
    ModulatedFilter(usize, usize, ModulatedFilterParams),
    Envelope(usize, EnvelopeParams),
}

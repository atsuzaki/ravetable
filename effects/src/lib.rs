use crate::filters::{
    IIRFilterStatePacket, IIRLowPassFilter, ModulatedFilter, ModulatedFilterStatePacket,
    StateVariableTPTFilter, StateVariableTPTFilterStatePacket,
};
use cpal::SampleRate;
use once_cell::sync::OnceCell;

pub mod adsr;
pub mod filters;
pub mod lfo;

// TODO: rather janky solution to avoid cyclic dependency with ravetable bin crate
//       rework a better option in the future.
pub static SAMPLE_RATE: OnceCell<SampleRate> = OnceCell::new();

pub fn get_sample_rate() -> f32 {
    SAMPLE_RATE.get().unwrap().0 as f32
}

pub fn set_effects_sample_rate(sample_rate: SampleRate) {
    SAMPLE_RATE.set(sample_rate).unwrap();
}

pub enum Effect {
    ModulatedFilter(ModulatedFilter),
    IIRFilter(IIRLowPassFilter), // TODO: IIRFilter should be more than lowpass
    StateVariablePTPFilter(StateVariableTPTFilter),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EffectStatePacket {
    ModulatedFilter(ModulatedFilterStatePacket),
    IIRFilter(IIRFilterStatePacket),
    StateVariablePTPFilter(StateVariableTPTFilterStatePacket),
}

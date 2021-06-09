use cpal::SampleRate;
use once_cell::sync::OnceCell;
use std::any::Any;

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

pub trait Effect {
    /// Applies effect to samples, mutating it in place
    fn process_samples(&mut self, samples_clock: u64, samples: &mut [f32]);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

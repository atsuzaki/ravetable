use std::any::Any;

pub mod filters;

pub trait Effect {
    /// Applies effect to samples, mutating it in place
    fn process_samples(&mut self, samples: &mut [f32]);
	fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

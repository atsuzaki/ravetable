pub mod filters;

pub trait Effect {
    /// Applies effect to samples, mutating it in place
    fn process_samples(&mut self, samples: &mut [f32]);
}

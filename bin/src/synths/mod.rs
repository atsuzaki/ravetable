use num_traits::FloatConst;

pub struct SineOscillator {
    gain: f32,
    current_angle: f32,
    angle_delta: f32,
}

impl SineOscillator {
    pub fn new(gain: f32, frequency: f32, sample_rate: f32) -> SineOscillator {
        let mut s = SineOscillator {
            gain,
            current_angle: 0.0,
            angle_delta: 0.0,
        };
        s.set_frequency(frequency, sample_rate);
        s
    }

    pub fn get_next_sample(&mut self) -> f32 {
        let current_sample = self.current_angle.sin();
        self.update_angle();
        current_sample * self.gain
    }

    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let cycles_per_sample = frequency / sample_rate;
        self.angle_delta = cycles_per_sample * f32::TAU();
    }

    fn update_angle(&mut self) {
        self.current_angle += self.angle_delta;
        if self.current_angle >= f32::TAU() {
            self.current_angle -= f32::TAU();
        }
    }
}

#[derive(Debug)]
pub struct RawWaveform {
    pub path: String,
    pub audio_buffer: Vec<f32>,
}

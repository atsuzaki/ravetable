// FROM HALFRATE:

// for wav in wavs.iter() {
// // hound read sample
// let mut reader = hound::WavReader::open(format!("test_wavs/{}", wav)).unwrap();
// let spec = reader.spec();
//
// let samples = reader.samples::<i16>().fold(vec![], |mut acc, s| {
// acc.push(s.unwrap() as f64);
// acc
// });
//
// let target_sample_rate = spec.sample_rate / 2;
// let freq = (target_sample_rate / 2) - 500;
//
// let mut filter = filters::IIRFilter::new_low_pass(spec.sample_rate as f64, freq as f64, 1.);
// let mut output_buffer = vec![0.0; samples.len()];
// filter.process_samples(&samples, &mut output_buffer);
//
// let spec = hound::WavSpec {
// channels: 1,
// sample_rate: target_sample_rate,
// bits_per_sample: 16,
// sample_format: hound::SampleFormat::Int, // Must use Int here instead of float for 16 bps
// };
//
// // Creating an output writer and write a sine wave to sine.wav
// let mut wav_writer =
// hound::WavWriter::create(format!("halfrate-{}", wav), spec).unwrap();
// for sample in samples.iter().step_by(2) {
// wav_writer.write_sample(*sample as i16).unwrap();
// }
// wav_writer.finalize().unwrap();
// }

#[derive(Debug)]
pub struct RawWaveform {
    pub path: String,
    pub audio_buffer: Vec<f64>,
}

pub fn load_waveform<P: AsRef<std::path::Path>>(path: P) -> RawWaveform {
    // Read sample with hound
    let mut reader = hound::WavReader::open(&path).unwrap();
    // let spec = reader.spec();

    // TODO: currently we only support samples with f32 bits per sample, genericize this to support more samples
    let samples = reader.samples::<f32>().fold(vec![], |mut acc, s| {
        acc.push(s.unwrap() as f64);
        acc
    });

    RawWaveform {
        path: path.as_ref().to_string_lossy().into_owned(),
        audio_buffer: samples
    }
}

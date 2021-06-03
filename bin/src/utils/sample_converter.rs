// use crate::InputWav;
// use samplerate::{ConverterType, Samplerate};
//
// pub fn load_waveform(filename: String, output_sample_rate: u32) -> InputWav {
//     let reader = hound::WavReader::open(&filename).unwrap();
//     let input_wav_spec = reader.spec();
//
//     let mut samples = reader.into_samples::<f32>();
//     let mut fsamples: Vec<f32> = samples.map(|f| f.unwrap()).collect();
//
//     if input_wav_spec.sample_rate != output_sample_rate {
//         println!("Converting sample");
//         // Instanciate a new converter.
//         let mut sample_rate_converter = Samplerate::new(
//             ConverterType::SincBestQuality,
//             input_wav_spec.sample_rate,
//             output_sample_rate,
//             input_wav_spec.channels as usize,
//         )
//         .unwrap();
//
//         // Resample the input from input sample rate to output sample rate
//         fsamples = sample_rate_converter.process_last(&fsamples).unwrap();
//     }
//
//     InputWav {
//         samples: Some(fsamples),
//         spec: input_wav_spec,
//         file_path: filename,
//     }
// }

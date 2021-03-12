use alloc::boxed::Box;
use core::f64::consts::PI;
use alloc::vec::Vec;

/// Creates a sine (sinus) wave function for a given frequency.
/// Don't forget to scale up the value to the audio resolution.
/// So far, amplitude is in interval `[-1; 1]`. The parameter
/// of the returned function is the point in time in seconds.
///
/// * `frequency` is in Hertz
pub fn sine_wave(frequency: f64) -> Box<dyn Fn(f64) -> f64> {
    Box::new(
        move |t| (t * frequency * 2.0 * PI).sin()
    )
}

/// See [`sine_wave_audio_data_multiple`]
pub fn sine_wave_audio_data(frequency: f64, sampling_rate: u32, duration_ms: u32) -> Vec<i16> {
    sine_wave_audio_data_multiple(&[frequency], sampling_rate, duration_ms)
}

/// Like [`sine_wave_audio_data`] but puts multiple sinus waves on top of each other.
/// Returns a audio signal encoded in 16 bit audio resolution which is the sum of
/// multiple sine waves on top of each other. The amplitudes will be scaled from
/// `[-1; 1]` to `[i16::min_value(); i16::max_value()]`
///
/// * `frequency` frequency in Hz for the sinus wave
/// * `sampling_rate` sampling rate, i.e. 44100Hz
/// * `duration_ms` duration of the audio data in milliseconds
pub fn sine_wave_audio_data_multiple(frequencies: &[f64], sampling_rate: u32, duration_ms: u32) -> Vec<i16> {
    if frequencies.is_empty() {
        return vec![];
    }

    // Generate all sine wave function
    let sine_waves = frequencies.iter()
        .map(|f| sine_wave(*f))
        .collect::<Vec<Box<dyn Fn(f64) -> f64>>>();

    // How many samples to generate with each sine wave function
    let sample_count = (sampling_rate as f64 * (duration_ms as f64 / 1000_f64)) as usize;

    // Calculate the final sine wave
    let mut sine_wave = Vec::with_capacity(sample_count);
    for i_sample in 0..sample_count {
        // t: time
        let t = (1.0/sampling_rate as f64) * i_sample as f64;

        // BEGIN: add sine waves
        let mut acc = 0.0;
        for i_sine_wave in 0..sine_waves.len() {
            acc += sine_waves[i_sine_wave](t);
        }
        // END: add sine waves

        // BEGIN: scale
        // times 0.6 to prevent to harsh clipping if multiple sinus waves are added above each other
        let acc = acc * i16::max_value() as f64 * 0.6;
        // END: scale

        // BEGIN: truncate in interval
        let acc = if acc > i16::max_value() as f64 {
            i16::max_value()
        } else if acc < i16::min_value() as f64 {
            i16::min_value()
        } else {
            acc as i16
        };
        // END: truncate in interval

        sine_wave.push(acc)
    }

    sine_wave
}

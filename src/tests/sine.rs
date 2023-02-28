/*
MIT License

Copyright (c) 2023 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
//! Module for generating synthetic sine waves.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::f32::consts::PI;

/// Creates a sine (sinus) wave function for a given frequency.
/// Don't forget to scale up the value to the audio resolution.
/// So far, amplitude is in interval `[-1; 1]`. The parameter
/// of the returned function is the point in time in seconds.
///
/// * `frequency` is in Hertz
pub fn sine_wave(frequency: f32) -> Box<dyn Fn(f32) -> f32> {
    Box::new(move |t| (t * frequency * 2.0 * PI).sin())
}

/// See [`sine_wave_audio_data_multiple`]
#[allow(dead_code)]
pub fn sine_wave_audio_data(frequency: f32, sampling_rate: u32, duration_ms: u32) -> Vec<i16> {
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
pub fn sine_wave_audio_data_multiple(
    frequencies: &[f32],
    sampling_rate: u32,
    duration_ms: u32,
) -> Vec<i16> {
    if frequencies.is_empty() {
        return vec![];
    }

    // Generate all sine wave function
    let sine_waves = frequencies
        .iter()
        .map(|f| sine_wave(*f))
        .collect::<Vec<Box<dyn Fn(f32) -> f32>>>();

    // How many samples to generate with each sine wave function
    let sample_count = (sampling_rate as f32 * (duration_ms as f32 / 1000.0)) as usize;

    // Calculate the final sine wave
    let mut sine_wave = Vec::with_capacity(sample_count);
    for i_sample in 0..sample_count {
        // t: time
        let t = (1.0 / sampling_rate as f32) * i_sample as f32;

        // BEGIN: add sine waves
        let mut acc = 0.0;
        for wave in &sine_waves {
            acc += wave(t);
        }
        // END: add sine waves

        // BEGIN: scale
        // times 0.1 to prevent to clipping if multiple sinus waves are added above each other
        let acc = acc * i16::MAX as f32 * 0.1;
        // END: scale

        // BEGIN: truncate in interval
        let acc = if acc > i16::MAX as f32 {
            i16::MAX
        } else if acc < i16::MIN as f32 {
            i16::MIN
        } else {
            acc as i16
        };
        // END: truncate in interval

        sine_wave.push(acc)
    }

    sine_wave
}

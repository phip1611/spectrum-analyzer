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
//! Several window functions which you can apply before doing the FFT.
//! For more information:
//! - <https://en.wikipedia.org/wiki/Window_function>
//! - <https://www.youtube.com/watch?v=dCeHOf4cJE0> (FFT and windowing by Texas Instruments)

use alloc::vec::Vec;
use core::f32::consts::PI;
// replacement for std functions like sin and cos in no_std-environments
use libm::cosf;

/// Applies a Hann window (<https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows>)
/// to an array of samples.
///
/// ## Return value
/// New vector with Hann window applied to the values.
#[must_use]
pub fn hann_window(samples: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    let samples_len_f32 = samples.len() as f32;
    for (i, sample) in samples.iter().enumerate() {
        let two_pi_i = 2.0 * PI * i as f32;
        let idontknowthename = cosf(two_pi_i / samples_len_f32);
        let multiplier = 0.5 * (1.0 - idontknowthename);
        windowed_samples.push(multiplier * sample)
    }
    windowed_samples
}

/// Applies a Hamming window (<https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows>)
/// to an array of samples.
///
/// ## Return value
/// New vector with Hann window applied to the values.
#[must_use]
pub fn hamming_window(samples: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    let samples_len_f32 = samples.len() as f32;
    for (i, sample) in samples.iter().enumerate() {
        let multiplier = 0.54 - (0.46 * (2.0 * PI * i as f32 / cosf(samples_len_f32 - 1.0)));
        windowed_samples.push(multiplier * sample)
    }
    windowed_samples
}

/// Applies a Blackman-Harris 4-term window (<https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window>)
/// to an array of samples.
///
/// ## Return value
/// New vector with Blackman-Harris 4-term window applied to the values.
#[must_use]
pub fn blackman_harris_4term(samples: &[f32]) -> Vec<f32> {
    // constants come from here:
    // https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window
    const ALPHA: [f32; 4] = [0.35875, -0.48829, 0.14128, -0.01168];

    blackman_harris_xterm(samples, &ALPHA)
}

/// Applies a Blackman-Harris 7-term window to an array of samples.
///
/// ## More information
/// * <https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window>
/// * <https://ieeexplore.ieee.org/document/940309>
/// * <https://dsp.stackexchange.com/questions/51095/seven-term-blackman-harris-window>
///
/// ## Return value
/// New vector with Blackman-Harris 7-term window applied to the values.
#[must_use]
pub fn blackman_harris_7term(samples: &[f32]) -> Vec<f32> {
    // constants come from here:
    // https://dsp.stackexchange.com/questions/51095/seven-term-blackman-harris-window
    const ALPHA: [f32; 7] = [
        0.271_051_4,
        -0.433_297_93,
        0.218_123,
        -0.065_925_45,
        0.010_811_742,
        -0.000_776_584_84,
        0.000_013_887_217,
    ];

    blackman_harris_xterm(samples, &ALPHA)
}

/// Applies a Blackman-Harris x-term window
/// (<https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window>)
/// to an array of samples. The x is specified by `alphas.len()`.
///
/// ## Return value
/// New vector with Blackman-Harris x-term window applied to the values.
#[must_use]
fn blackman_harris_xterm(samples: &[f32], alphas: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());

    let samples_len_f32 = samples.len() as f32;

    for sample in samples.iter() {
        // Will result in something like that:
        /* ALPHA0
            + ALPHA1 * ((2.0 * PI * *samples[i])/samples_len_f32).cos()
            + ALPHA2 * ((4.0 * PI * *samples[i])/samples_len_f32).cos()
            + ALPHA3 * ((6.0 * PI * *samples[i])/samples_len_f32).cos()
        */

        let mut acc = 0.0;
        for (alpha_i, alpha) in alphas.iter().enumerate() {
            // in 1. iter. 0PI, then 2PI, then 4 PI, then 6 PI
            let two_pi_iteration = 2.0 * alpha_i as f32 * PI;
            let cos = cosf((two_pi_iteration * sample) / samples_len_f32);
            acc += alpha * cos;
        }

        windowed_samples.push(acc)
    }

    windowed_samples
}

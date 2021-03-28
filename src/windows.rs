/*
MIT License

Copyright (c) 2021 Philipp Schuster

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
//! - https://en.wikipedia.org/wiki/Window_function
//! - https://www.youtube.com/watch?v=dCeHOf4cJE0 (FFT and windowing by Texas Instruments)

use alloc::vec::Vec;
use core::f32::consts::PI;
// replacement for std functions like sin and cos in no_std-environments
use libm::cosf;

/*/// Describes what window function should be applied to
/// the `samples` parameter of [`crate::samples_fft_to_spectrum`]
/// should be applied before the FFT starts. See
/// https://en.wikipedia.org/wiki/Window_function for more
/// resources.
pub enum WindowFn {

}*/

/// Applies a Hann window (https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows)
/// to an array of samples.
///
/// ## Return value
/// New vector with Hann window applied to the values.
pub fn hann_window(samples: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    let samples_len_f32 = samples.len() as f32;
    for i in 0..samples.len() {
        let two_pi_i = 2.0 * PI * i as f32;
        let idontknowthename = cosf(two_pi_i / samples_len_f32);
        let multiplier = 0.5 * (1.0 - idontknowthename);
        windowed_samples.push(multiplier * samples[i])
    }
    windowed_samples
}

/// Applies a Hamming window (https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows)
/// to an array of samples.
///
/// ## Return value
/// New vector with Hann window applied to the values.
pub fn hamming_window(samples: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    let samples_len_f32 = samples.len() as f32;
    for i in 0..samples.len() {
        let multiplier = 0.54 - (0.46 * (2.0 * PI * i as f32 / cosf(samples_len_f32 - 1.0)));
        windowed_samples.push(multiplier * samples[i])
    }
    windowed_samples
}

/// Applies a Blackman-Harris 4-term window (https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window)
/// to an array of samples.
///
/// ## Return value
/// New vector with Blackman-Harris 4-term window applied to the values.
pub fn blackman_harris_4term(samples: &[f32]) -> Vec<f32> {
    // constants come from here:
    // https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window
    const ALPHA: [f32; 4] = [0.35875, -0.48829, 0.14128, -0.01168];

    blackman_harris_xterm(samples, &ALPHA)
}

/// Applies a Blackman-Harris 7-term window to an array of samples.
///
/// ## More information
/// * https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window
/// * https://ieeexplore.ieee.org/document/940309
/// * https://dsp.stackexchange.com/questions/51095/seven-term-blackman-harris-window
///
/// ## Return value
/// New vector with Blackman-Harris 7-term window applied to the values.
pub fn blackman_harris_7term(samples: &[f32]) -> Vec<f32> {
    // constants come from here:
    // https://dsp.stackexchange.com/questions/51095/seven-term-blackman-harris-window
    const ALPHA: [f32; 7] = [
        0.27105140069342,
        -0.43329793923448,
        0.21812299954311,
        -0.06592544638803,
        0.01081174209837,
        -0.00077658482522,
        0.00001388721735,
    ];

    blackman_harris_xterm(samples, &ALPHA)
}

/// Applies a Blackman-Harris x-term window
/// (https://en.wikipedia.org/wiki/Window_function#Blackman%E2%80%93Harris_window)
/// to an array of samples. The x is specified by `alphas.len()`.
///
/// ## Return value
/// New vector with Blackman-Harris x-term window applied to the values.
fn blackman_harris_xterm(samples: &[f32], alphas: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());

    let samples_len_f32 = samples.len() as f32;

    for i in 0..samples.len() {
        let mut acc = 0.0;

        // Will result in something like that:
        /* ALPHA0
            + ALPHA1 * ((2.0 * PI * *samples[i])/samples_len_f32).cos()
            + ALPHA2 * ((4.0 * PI * *samples[i])/samples_len_f32).cos()
            + ALPHA3 * ((6.0 * PI * *samples[i])/samples_len_f32).cos()
        */

        for alpha_i in 0..alphas.len() {
            // in 1. iter. 0PI, then 2PI, then 4 PI, then 6 PI
            let two_pi_iteration = 2.0 * alpha_i as f32 * PI;
            let sample = samples[i];
            let cos = cosf((two_pi_iteration * sample) / samples_len_f32);
            acc += alphas[alpha_i] * cos;
        }

        windowed_samples.push(acc)
    }

    windowed_samples
}

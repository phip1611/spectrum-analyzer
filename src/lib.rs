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
//! A simple and fast `no_std` library to get the frequency spectrum of a digital signal
//! (e.g. audio) using FFT. It follows the KISS principle and consists of simple building
//! blocks/optional features.
//!
//! In short, this is a convenient wrapper around an FFT implementation. You choose the
//! implementation at compile time via Cargo features. As of version 0.4.0 this uses
//! "microfft"-crate.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(missing_crate_level_docs)]
#![deny(missing_doc_code_examples)]
#![deny(missing_docs)]
#![cfg_attr(test, no_std)]

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

// We use alloc crate, because this is no_std
// The macros are only needed when we test
#[cfg_attr(test, macro_use)]
extern crate alloc;

use alloc::vec::Vec;

use crate::error::SpectrumAnalyzerError;
use crate::fft::{Complex32, Fft, FftImpl};
pub use crate::frequency::{Frequency, FrequencyValue};
pub use crate::limit::FrequencyLimit;
pub use crate::spectrum::{ComplexSpectrumScalingFunction, FrequencySpectrum};
use core::convert::identity;

pub mod error;
mod fft;
mod frequency;
mod limit;
pub mod scaling;
mod spectrum;
pub mod windows;

// test module for large "integration"-like tests
#[cfg(test)]
mod tests;

/// Definition of a simple function that gets applied on each frequency magnitude
/// in the spectrum. This is easier to write, especially for Rust beginners.
/// Everything that can be achieved with this, can also be achieved with parameter
/// `total_scaling_fn`.
///
/// The scaling only affects the value/amplitude of the frequency
/// but not the frequency itself.
pub type SimpleSpectrumScalingFunction<'a> = &'a dyn Fn(f32) -> f32;

/// Takes an array of samples (length must be a power of 2),
/// e.g. 2048, applies an FFT (using the specified FFT implementation) on it
/// and returns all frequencies with their volume/magnitude.
///
/// By default, no normalization/scaling is done at all and the results,
/// i.e. the frequency magnitudes/amplitudes/values are the raw result from
/// the FFT algorithm, except that complex numbers are transformed
/// to their magnitude.
///
/// * `samples` raw audio, e.g. 16bit audio data but as f32.
///             You should apply an window function (like Hann) on the data first.
///             The final frequency resolution is `sample_rate / (N / 2)`
///             e.g. `44100/(16384/2) == 5.383Hz`, i.e. more samples =>
///             better accuracy/frequency resolution.
/// * `sampling_rate` sampling_rate, e.g. `44100 [Hz]`
/// * `frequency_limit` Frequency limit. See [`FrequencyLimit´]
/// * `per_element_scaling_fn` See [`crate::SimpleSpectrumScalingFunction`] for details.
///                            This is easier to write, especially for Rust beginners. Everything
///                            that can be achieved with this, can also be achieved with
///                            parameter `total_scaling_fn`.
///                            See [`crate::scaling`] for example implementations.
/// * `total_scaling_fn` See [`crate::spectrum::SpectrumTotalScaleFunctionFactory`] for details.
///                      See [`crate::scaling`] for example implementations.
///
/// ## Returns value
/// New object of type [`FrequencySpectrum`].
///
/// ## Panics
/// * When `samples` contains NaN or infinite values (regarding f32/float).
/// * When `samples.len()` isn't a power of two and `samples.len() > 4096`
///   (restriction by `microfft`-crate)
pub fn samples_fft_to_spectrum(
    samples: &[f32],
    sampling_rate: u32,
    frequency_limit: FrequencyLimit,
    per_element_scaling_fn: Option<SimpleSpectrumScalingFunction>,
    total_scaling_fn: Option<ComplexSpectrumScalingFunction>,
) -> Result<FrequencySpectrum, SpectrumAnalyzerError> {
    // everything below two samples is unreasonable
    if samples.len() < 2 {
        return Err(SpectrumAnalyzerError::TooFewSamples);
    }
    // do several checks on input data
    if samples.iter().any(|x| x.is_nan()) {
        return Err(SpectrumAnalyzerError::NaNValuesNotSupported);
    }
    if samples.iter().any(|x| x.is_infinite()) {
        return Err(SpectrumAnalyzerError::InfinityValuesNotSupported);
    }
    if !is_power_of_two(samples.len()) {
        return Err(SpectrumAnalyzerError::SamplesLengthNotAPowerOfTwo);
    }
    let max_detectable_frequency = sampling_rate as f32 / 2.0;
    // verify frequency limit: unwrap error or else ok
    let _ = frequency_limit
        .verify(max_detectable_frequency)
        .map_err(|e| SpectrumAnalyzerError::InvalidFrequencyLimit(e))?;

    // With FFT we transform an array of time-domain waveform samples
    // into an array of frequency-domain spectrum samples
    // https://www.youtube.com/watch?v=z7X6jgFnB6Y

    // FFT result has same length as input
    // (but when we interpret the result, we don't need all indices)

    // applies the f32 samples onto the FFT algorithm implementation
    // chosen at compile time (via Cargo feature).
    // If a complex FFT implementation was chosen, this will internally
    // transform all data to Complex numbers.
    let buffer = FftImpl::fft_apply(samples);

    // This function:
    // 1) calculates the corresponding frequency of each index in the FFT result
    // 2) filters out unwanted frequencies
    // 3) calculates the magnitude (absolute value) at each frequency index for each complex value
    // 4) optionally scales the magnitudes
    // 5) collects everything into the struct "FrequencySpectrum"
    fft_result_to_spectrum(
        samples.len(),
        &buffer,
        sampling_rate,
        frequency_limit,
        per_element_scaling_fn,
        total_scaling_fn,
    )
}

/// Transforms the FFT result into the spectrum by calculating the corresponding frequency of each
/// FFT result index and optionally calculating the magnitudes of the complex numbers if a complex
/// FFT implementation is chosen.
///
/// ## Parameters
/// * `samples_len` Length of samples. This is a dedicated field because it can't always be
///                 derived from `fft_result.len()`. There are for example differences for
///                  `fft_result.len()` in real and complex FFT.
/// * `fft_result` Result buffer from FFT. Has the same length as the samples array.
/// * `sampling_rate` sampling_rate, e.g. `44100 [Hz]`
/// * `frequency_limit` Frequency limit. See [`FrequencyLimit´]
/// * `per_element_scaling_fn` Optional per element scaling function, e.g. `20 * log(x)`.
///                            To see where this equation comes from, check out
///                            this paper:
///                            https://www.sjsu.edu/people/burford.furman/docs/me120/FFT_tutorial_NI.pdf
/// * `total_scaling_fn` See [`crate::spectrum::SpectrumTotalScaleFunctionFactory`].
///
/// ## Return value
/// New object of type [`FrequencySpectrum`].
#[inline(always)]
fn fft_result_to_spectrum(
    samples_len: usize,
    fft_result: &[Complex32],
    sampling_rate: u32,
    frequency_limit: FrequencyLimit,
    per_element_scaling_fn: Option<&dyn Fn(f32) -> f32>,
    total_scaling_fn: Option<ComplexSpectrumScalingFunction>,
) -> Result<FrequencySpectrum, SpectrumAnalyzerError> {
    let maybe_min = frequency_limit.maybe_min();
    let maybe_max = frequency_limit.maybe_max();

    let frequency_resolution = fft_calc_frequency_resolution(sampling_rate, samples_len as u32);

    // collect frequency => frequency value in Vector of Pairs/Tuples
    let frequency_vec = fft_result
        .into_iter()
        // See https://stackoverflow.com/a/4371627/2891595 for more information as well as
        // https://www.gaussianwaves.com/2015/11/interpreting-fft-results-complex-dft-frequency-bins-and-fftshift/
        //
        // The indices 0 to N/2 (inclusive) are usually the most relevant. Although, index
        // N/2-1 is declared as the last useful one on stackoverflow (because in typical applications
        // Nyquist-frequency + above are filtered out), we include everything here.
        // with 0..=(samples_len / 2) (inclusive) we get all frequencies from 0 to Nyquist theorem.
        //
        // Indices (samples_len / 2)..len() are mirrored/negative. You can also see this here:
        // https://www.gaussianwaves.com/gaussianwaves/wp-content/uploads/2015/11/realDFT_complexDFT.png
        .take(FftImpl::fft_relevant_res_samples_count(samples_len))
        // to (index, fft-result)-pairs
        .enumerate()
        // calc index => corresponding frequency
        .map(|(fft_index, fft_result)| {
            (
                // Calculate corresponding frequency of each index of FFT result.
                //
                // Explanation for the algorithm:
                // https://stackoverflow.com/questions/4364823/
                //
                // N complex samples          : [0], [1], [2], [3], ... , ..., [2047] => 2048 samples for example
                //   (Or N real samples packed
                //   into N/2 complex samples
                //   (real FFT algorithm))
                // Complex FFT Result         : [0], [1], [2], [3], ... , ..., [2047]
                // Relevant part of FFT Result: [0], [1], [2], [3], ... , [1024]      => indices 0 to N/2 (inclusive) are important
                //                               ^                         ^
                // Frequency                  : 0Hz, .................... Sampling Rate/2 => "Nyquist frequency"
                //                              0Hz is also called        (e.g. 22050Hz for 44100Hz sampling rate)
                //                              "DC Component"
                //
                // frequency step/resolution is for example: 1/2048 * 44100 = 21.53 Hz
                //                                             2048 samples, 44100 sample rate
                //
                // equal to: 1.0 / samples_len as f32 * sampling_rate as f32
                fft_index as f32 * frequency_resolution,
                // in this .map() step we do nothing with this yet
                fft_result,
            )
        })
        // #######################
        // ### BEGIN filtering: results in lower calculation and memory overhead!
        // check lower bound frequency (inclusive)
        .filter(|(fr, _fft_result)| {
            if let Some(min_fr) = maybe_min {
                // inclusive!
                // attention: due to the frequency resolution, we do not necessarily hit
                //            exactly the frequency, that a user requested
                //            e.g. 1416.8 < limit < 1425.15
                *fr >= min_fr
            } else {
                true
            }
        })
        // check upper bound frequency (inclusive)
        .filter(|(fr, _fft_result)| {
            if let Some(max_fr) = maybe_max {
                // inclusive!
                // attention: due to the frequency resolution, we do not necessarily hit
                //            exactly the frequency, that a user requested
                //            e.g. 1416.8 < limit < 1425.15
                *fr <= max_fr
            } else {
                true
            }
        })
        // ### END filtering
        // #######################
        // FFT result is always complex: calc magnitude
        //   sqrt(re*re + im*im) (re: real part, im: imaginary part)
        .map(|(fr, complex_res)| (fr, complex_to_magnitude(&complex_res)))
        // apply optionally scale function
        .map(|(fr, val)| (fr, per_element_scaling_fn.unwrap_or(&identity)(val)))
        // transform to my thin convenient orderable f32 wrappers
        .map(|(fr, val)| (Frequency::from(fr), FrequencyValue::from(val)))
        // collect all into an sorted vector (from lowest frequency to highest)
        .collect::<Vec<(Frequency, FrequencyValue)>>();

    // create spectrum object
    let spectrum = FrequencySpectrum::new(frequency_vec, frequency_resolution);

    // optionally scale
    if let Some(total_scaling_fn) = total_scaling_fn {
        spectrum.apply_complex_scaling_fn(total_scaling_fn)
    }

    Ok(spectrum)
}

/// Calculate the frequency resolution of the FFT. It is determined by the sampling rate
/// in Hertz and N, the number of samples given into the FFT. With the frequency resolution,
/// we can determine the corresponding frequency of each index in the FFT result buffer.
///
/// For "real FFT" implementations
///
/// ## Parameters
/// * `samples_len` Number of samples put into the FFT
/// * `sampling_rate` sampling_rate, e.g. `44100 [Hz]`
///
/// ## Return value
/// Frequency resolution in Hertz.
///
/// ## More info
/// * https://www.researchgate.net/post/How-can-I-define-the-frequency-resolution-in-FFT-And-what-is-the-difference-on-interpreting-the-results-between-high-and-low-frequency-resolution
/// * https://stackoverflow.com/questions/4364823/
#[inline(always)]
fn fft_calc_frequency_resolution(sampling_rate: u32, samples_len: u32) -> f32 {
    sampling_rate as f32 / samples_len as f32
}

/// Maps a [`Complex32`] to it's magnitude as `f32`. This is done
/// by calculating `sqrt(re*re + im*im)`. This is required to convert
/// the complex FFT result back to real values.
///
/// ## Parameters
/// * `val` A single value from the FFT output buffer of type [`Complex32`].
fn complex_to_magnitude(val: &Complex32) -> f32 {
    // calculates sqrt(re*re + im*im), i.e. magnitude of complex number
    let sum = val.re * val.re + val.im * val.im;
    let sqrt = libm::sqrtf(sum);
    debug_assert!(sqrt != f32::NAN, "sqrt is NaN!");
    sqrt
}

// idea from https://stackoverflow.com/questions/600293/how-to-check-if-a-number-is-a-power-of-2
fn is_power_of_two(num: usize) -> bool {
    num != 0 && ((num & (num - 1)) == 0)
}

// tests module for small unit tests

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_is_power_of_two() {
        assert!(!is_power_of_two(0));
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(!is_power_of_two(3));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(256));
    }
}

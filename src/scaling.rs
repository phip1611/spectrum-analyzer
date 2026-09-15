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
//! This module contains convenient public transform functions that you can use
//! as parameters in [`samples_fft_to_spectrum`] for scaling the frequency value
//! (the FFT result).
//!
//! They act as "idea/inspiration". Feel free to create your own derivation
//! from them. To chain two of them, write a closure:
//!
//! ```
//! use spectrum_analyzer::scaling::{divide_by_N, scale_20_times_log10};
//! let scaling_fn = |val, stats: &_| scale_20_times_log10(divide_by_N(val, stats), stats);
//! ```
//!
//! [`samples_fft_to_spectrum`]: crate::samples_fft_to_spectrum

/// Helper struct for [`SpectrumScalingFunction`] that is passed into the
/// scaling function together with the current frequency value.
///
/// This structure can be used to scale each value. All properties reference the
/// current data of a [`FrequencySpectrum`].
///
/// This uses `f32` in favor of [`FrequencyValue`] because the latter led to
/// some implementation problems.
///
/// [`FrequencySpectrum`]: crate::FrequencySpectrum
/// [`FrequencyValue`]: crate::FrequencyValue
#[derive(Debug)]
pub struct SpectrumDataStats {
    /// Minimal frequency value in spectrum.
    pub min: f32,
    /// Maximum frequency value in spectrum.
    pub max: f32,
    /// Average frequency value in spectrum.
    pub average: f32,
    /// Median frequency value in spectrum.
    pub median: f32,
    /// Number of samples (`samples.len()`), not the number of values in the
    /// spectrum (which can be smaller due to a frequency limit).
    pub n: f32,
}

/// Describes the type for a function that scales/normalizes the data inside
/// [`FrequencySpectrum`].
///
/// The scaling only affects the value of the frequency, but not the
/// frequency itself. It is applied to every single element.
///
/// A scaling function can be used for example to subtract the minimum (`min`)
/// from each value. It is optional to use the second parameter
/// [`SpectrumDataStats`], which describes the spectrum before the function is
/// applied to it.
///
/// The type works with static functions as well as dynamically created
/// closures.
///
/// You must take care of, that you don't have division by zero in your function
/// or that the result is NaN or Infinity (regarding IEEE-754). If the result
/// is NaN or Infinity, the library will return `Err`.
///
/// This uses `f32` in favor of [`FrequencyValue`] because the latter led to
/// some implementation problems.
///
/// [`FrequencySpectrum`]: crate::FrequencySpectrum
/// [`FrequencyValue`]: crate::FrequencyValue
pub type SpectrumScalingFunction = dyn Fn(f32, &SpectrumDataStats) -> f32;

/// Lower bound for the input of [`scale_20_times_log10`], i.e., `-100 dB`.
const DB_FLOOR: f32 = 1e-5;

/// Converts each value to decibels: `20 * log10(value)`.
///
/// A value of `1.0` becomes `0 dB`. Unscaled values grow with the number of
/// samples (see [`crate::samples_fft_to_spectrum`]), so the absolute levels
/// depend on `N` and on the input range. For levels relative to a full-scale
/// sine wave (dBFS), scale to amplitudes first, in a separate call to
/// `apply_scaling_fn`.
///
/// Values below `1e-5` are clamped, so the result is never below `-100 dB`
/// and silence stays at the bottom of the scale.
///
/// This scaling is quite common, you can find more information for example
/// here:
/// <https://www.sjsu.edu/people/burford.furman/docs/me120/FFT_tutorial_NI.pdf>
///
/// ## Usage
/// ```rust
///use spectrum_analyzer::{samples_fft_to_spectrum, scaling, FrequencyLimit};
///let window = [0.0, 0.1, 0.2, 0.3]; // add real data here
///let spectrum = samples_fft_to_spectrum(
///     &window,
///     44100,
///     FrequencyLimit::All,
///     Some(&scaling::scale_20_times_log10),
/// );
/// ```
/// Function is of type [`SpectrumScalingFunction`].
#[must_use]
pub fn scale_20_times_log10(fr_val: f32, _stats: &SpectrumDataStats) -> f32 {
    debug_assert!(!fr_val.is_infinite());
    debug_assert!(!fr_val.is_nan());
    debug_assert!(fr_val >= 0.0);
    // Clamping keeps silence below every other value (0 dB would not).
    20.0 * libm::log10f(fr_val.max(DB_FLOOR))
}

/// Divides each value by the maximum, so that the loudest frequency becomes
/// `1.0` and every other keeps its ratio to it.
///
/// The smallest value only becomes `0.0` if it already was `0.0`; the values
/// are not stretched over the whole interval. All of them must be positive or
/// zero, which holds for magnitudes but not for the output of
/// [`scale_20_times_log10`]. If the maximum is `0.0`, all values become `0.0`.
///
/// Function is of type [`SpectrumScalingFunction`].
#[must_use]
pub fn scale_to_zero_to_one(fr_val: f32, stats: &SpectrumDataStats) -> f32 {
    debug_assert!(!fr_val.is_infinite());
    debug_assert!(!fr_val.is_nan());
    debug_assert!(fr_val >= 0.0);
    if stats.max != 0.0 {
        fr_val / stats.max
    } else {
        0.0
    }
}

/// Divides each value by `N`, the number of samples.
///
/// This makes spectra of different lengths comparable. A sine wave with
/// amplitude `A` on a bin frequency then shows up as `A / 2` (times the
/// window's coherent gain), see [`crate::samples_fft_to_spectrum`].
#[allow(non_snake_case)]
#[must_use]
pub fn divide_by_N(fr_val: f32, stats: &SpectrumDataStats) -> f32 {
    debug_assert!(!fr_val.is_infinite());
    debug_assert!(!fr_val.is_nan());
    debug_assert!(fr_val >= 0.0);
    if stats.n == 0.0 {
        fr_val
    } else {
        fr_val / stats.n
    }
}

/// Like [`divide_by_N`] but divides each value by `sqrt(N)`.
///
/// This is the normalization that preserves the energy of the signal, which
/// `rustfft` recommends for a forward and inverse transform pair. The values
/// still grow with `sqrt(N)`, so for comparing spectra of different lengths
/// use [`divide_by_N`] instead.
/// See <https://docs.rs/rustfft/latest/rustfft/#normalization>
#[allow(non_snake_case)]
#[must_use]
pub fn divide_by_N_sqrt(fr_val: f32, stats: &SpectrumDataStats) -> f32 {
    debug_assert!(!fr_val.is_infinite());
    debug_assert!(!fr_val.is_nan());
    debug_assert!(fr_val >= 0.0);
    if stats.n == 0.0 {
        fr_val
    } else {
        // https://docs.rs/rustfft/latest/rustfft/#normalization
        fr_val / libm::sqrtf(stats.n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_scale_to_zero_to_one() {
        let data = vec![0.0_f32, 1.1, 2.2, 3.3, 4.4, 5.5];
        let stats = SpectrumDataStats {
            min: data[0],
            max: data[data.len() - 1],
            average: data.iter().sum::<f32>() / data.len() as f32,
            median: (2.2 + 3.3) / 2.0,
            n: data.len() as f32,
        };
        // check that type matches
        let scaling_fn: &SpectrumScalingFunction = &scale_to_zero_to_one;
        let scaled_data = data
            .into_iter()
            .map(|x| scaling_fn(x, &stats))
            .collect::<Vec<_>>();
        let expected = [0.0_f32, 0.2, 0.4, 0.6, 0.8, 1.0];
        for (expected_val, actual_val) in expected.iter().zip(scaled_data.iter()) {
            float_cmp::approx_eq!(f32, *expected_val, *actual_val, ulps = 3);
        }
    }

    #[test]
    fn test_scale_20_times_log10() {
        let stats = SpectrumDataStats {
            min: 0.0,
            max: 10.0,
            average: 0.0,
            median: 0.0,
            n: 4.0,
        };
        let db = |val: f32| scale_20_times_log10(val, &stats);
        assert!(float_cmp::approx_eq!(f32, db(1.0), 0.0, epsilon = 1e-4));
        assert!(float_cmp::approx_eq!(f32, db(10.0), 20.0, epsilon = 1e-4));
        assert!(float_cmp::approx_eq!(f32, db(0.0), -100.0, epsilon = 1e-3));
        // silence must stay below every other value
        assert!(db(0.0) < db(0.5) && db(0.5) < db(1.0));
    }

    /// A closure replaces the removed `combined()`: it can chain the
    /// functions and, unlike `combined()`, capture its environment.
    #[test]
    fn test_chaining_with_a_closure() {
        let stats = SpectrumDataStats {
            min: 0.0,
            max: 10.0,
            average: 5.0,
            median: 5.0,
            n: 4.0,
        };
        let scaling_fn = |val, stats: &_| scale_20_times_log10(divide_by_N(val, stats), stats);
        let _: &SpectrumScalingFunction = &scaling_fn;
        // 10.0 / 4 = 2.5 -> 20 * log10(2.5)
        assert!(float_cmp::approx_eq!(
            f32,
            scaling_fn(stats.max, &stats),
            7.9588,
            epsilon = 1e-3
        ));
    }
}

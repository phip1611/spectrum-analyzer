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
//! as parameters in [`crate::samples_fft_to_spectrum`] for scaling the
//! frequency value (the FFT result). They act as "idea/inspiration". Feel free
//! to either compose them or create your own derivation from them.

use alloc::boxed::Box;

/// Helper struct for [`SpectrumScalingFunction`] that is passed into the
/// scaling function together with the current frequency value. This structure
/// can be used to scale each value. All properties reference the current data
/// of a [`crate::spectrum::FrequencySpectrum`].
///
/// This uses `f32` in favor of [`crate::FrequencyValue`] because the latter led to
/// some implementation problems.
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
    /// Number of samples (`samples.len()`). Already casted to f32, to avoid
    /// repeatedly casting in a loop for each value.
    pub n: f32,
}

/// Describes the type for a function that scales/normalizes the data inside [`crate::FrequencySpectrum`].
/// The scaling only affects the value/amplitude of the frequency, but not the frequency itself.
/// It is applied to every single element.
///
/// A scaling function can be used for example to subtract the minimum (`min`) from each value.
/// It is optional to use the second parameter [`SpectrumDataStats`].
/// and the type works with static functions as well as dynamically created closures.
///
/// You must take care of, that you don't have division by zero in your function or
/// that the result is NaN or Infinity (regarding IEEE-754). If the result is NaN or Infinity,
/// the library will return `Err`.
///
/// This uses `f32` in favor of [`crate::FrequencyValue`] because the latter led to
/// some implementation problems.
pub type SpectrumScalingFunction = dyn Fn(f32, &SpectrumDataStats) -> f32;

/// Calculates the base 10 logarithm of each frequency magnitude and
/// multiplies it with 20. This scaling is quite common, you can
/// find more information for example here:
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
    if fr_val == 0.0 {
        0.0
    } else {
        20.0 * libm::log10f(fr_val)
    }
}

/// Scales each frequency value/amplitude in the spectrum to interval `[0.0; 1.0]`.
/// Function is of type [`SpectrumScalingFunction`]. Expects that [`SpectrumDataStats::min`] is
/// not negative.
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

/// Divides each value by N. Several resources recommend that the FFT result should be divided
/// by the length of samples, so that values of different samples lengths are comparable.
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

/// Like [`divide_by_N`] but divides each value by `sqrt(N)`. This is the recommended scaling
/// in the `rustfft` documentation (but is generally applicable).
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

/// Combines several scaling functions into a new single one.
///
/// Currently there is the limitation that the functions need to have
/// a `'static` lifetime. This will be fixed if someone needs this.
///
/// # Example
/// ```
/// use spectrum_analyzer::scaling::{combined, divide_by_N, scale_20_times_log10};
/// let fncs = combined(&[&divide_by_N, &scale_20_times_log10]);
/// ```
pub fn combined(fncs: &'static [&SpectrumScalingFunction]) -> Box<SpectrumScalingFunction> {
    Box::new(move |val, stats| {
        let mut val = val;
        for fnc in fncs {
            val = fnc(val, stats);
        }
        val
    })
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
        let expected = vec![0.0_f32, 0.2, 0.4, 0.6, 0.8, 1.0];
        for (expected_val, actual_val) in expected.iter().zip(scaled_data.iter()) {
            float_cmp::approx_eq!(f32, *expected_val, *actual_val, ulps = 3);
        }
    }

    // make sure this compiles
    #[test]
    fn test_combined_compiles() {
        let _combined_static = combined(&[&scale_20_times_log10, &divide_by_N, &divide_by_N_sqrt]);

        // doesn't compile yet.. fix this once someone requests it
        /*let closure_scaling_fnc = |fr_val: f32, _stats: &SpectrumDataStats| {
           0.0
        };

        let _combined_dynamic = combined(&[
            &scale_20_times_log10,
            &divide_by_N,
            &divide_by_N_sqrt,
            &closure_scaling_fnc,
        ]);*/
    }
}

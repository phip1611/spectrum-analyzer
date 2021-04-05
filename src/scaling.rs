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
//! This module contains convenient public transform functions that you can use
//! as parameters in [`crate::samples_fft_to_spectrum`].

/// Practical implementations for [`crate::SimpleSpectrumScalingFunction`].
pub mod basic {

    /// Calculates the base 10 logarithm of each frequency magnitude and
    /// multiplies it with 20. This scaling is quite common, you can
    /// find more information for example here:
    /// https://www.sjsu.edu/people/burford.furman/docs/me120/FFT_tutorial_NI.pdf
    ///
    /// ## Usage
    /// ```rust
    ///use spectrum_analyzer::{samples_fft_to_spectrum, scaling, FrequencyLimit};
    ///let window = [0.0, 0.1, 0.2, 0.3]; // add real data here
    ///let spectrum = samples_fft_to_spectrum(
    ///     &window,
    ///     44100,
    ///     FrequencyLimit::All,
    ///     Some(&scaling::basic::scale_20_times_log10),
    ///     None,
    /// );
    /// ```
    pub fn scale_20_times_log10(frequency_magnitude: f32) -> f32 {
        20.0 * libm::log10f(frequency_magnitude)
    }
}

/// Practical implementations for [`crate::spectrum::ComplexSpectrumScalingFunction`].
pub mod complex {
    use crate::ComplexSpectrumScalingFunction;
    use alloc::boxed::Box;

    /// Returns a function factory that generates a function that scales
    /// each frequency value/amplitude in the spectrum to interval `[0.0; 1.0]`.
    pub fn scale_to_zero_to_one() -> ComplexSpectrumScalingFunction {
        Box::new(move |_min: f32, max: f32, _average: f32, _median: f32| Box::new(move |x| x / max))
    }
}

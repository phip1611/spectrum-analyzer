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
//! Abstraction over FFT implementation. This is mainly necessary because we might have
//! `no_std`/`std` implementations as well as real/complex implementations and especially
//! `real` FFT implementations might need a few adjustments.
//!
//! This crate compiles only iff exactly one feature, i.e. one FFT implementation, is activated.
//!
//! ## What FFT implementation to choose?
//! "microfft-real" should be in any way the fastest implementation and fine in any case.
//! I added multiple implementations primarily for educational reasons to myself to learn
//! differences between real and complex FFT. As of release 0.5.0 there is no valid case
//! why you should switch to another FFT implementation. They are primarily useful to me
//! during development to test different FFT implementations and see if my code is correct.
//!
//! ## Tips for development/testing
//! Usually I do all tests against "rustfft" because it is the
//! most actively developed implementation and also the fastest and most accurate, at least
//! in `std`-environments on modern processors. To test other implementations I usually
//! plot the results using the function
//! `crate::tests::test_spectrum_and_visualize_sine_waves_50_1000_3777hz` by invoking it
//! with different features (FFT implementations) enabled.

#[cfg(feature = "microfft-complex")]
mod microfft_complex;
#[cfg(feature = "microfft-complex")]
pub use microfft_complex::*;

#[cfg(feature = "microfft-real")]
mod microfft_real;
#[cfg(feature = "microfft-real")]
pub use microfft_real::*;

#[cfg(feature = "rustfft-complex")]
mod rustfft_complex;
#[cfg(feature = "rustfft-complex")]
pub use rustfft_complex::*;

use alloc::vec::Vec;

/// Abstraction over different FFT implementations. This is necessary because this crate
/// uses different compile time features to exchange the FFT implementation, i.e. real or complex.
/// Each of them operates on possibly different "num-complex"-versions for example.
pub(crate) trait Fft<ComplexType> {
    /// Applies the FFT on the given implementation. If necessary, the data is converted to a
    /// complex number first. The resulting vector contains complex numbers without any
    /// normalization/scaling. Usually you calc the magnitude of each complex number on the
    /// resulting vector to get the amplitudes of the frequencies in the next step.
    ///
    /// ## Parameters
    /// * `samples` samples for FFT. Length MUST be a power of 2 for FFT, e.g. 1024 or 4096!
    ///
    /// ## Return
    /// Vector of FFT results.
    fn fft_apply(samples: &[f32]) -> Vec<ComplexType>;

    /// Returns the relevant results of the FFT result. For complex FFT this is
    /// `N/2 + 1`, i.e. indices `0..=N/2` (inclusive end) are relevant. Real FFT
    /// implementations might be different here, because they may only have
    /// `N/2` results.
    ///
    /// The return value of this multiplied with `frequency_resolution` usually
    /// refers to the Nyquist frequency.
    ///
    /// For complex FFT we don't need the second half because it refers to
    /// negative frequency values (mirrored to first half with pos frequency values),
    /// therefore we skip it; the return value is smaller than `complex_samples.len()`.
    ///
    /// ## More info
    /// * https://www.researchgate.net/post/How-can-I-define-the-frequency-resolution-in-FFT-And-what-is-the-difference-on-interpreting-the-results-between-high-and-low-frequency-resolution
    /// * https://stackoverflow.com/questions/4364823/
    ///
    /// ## Parameters
    /// * `samples_len` Number of samples put into the FFT
    ///
    /// ## Return value
    /// Number of relevant samples.
    fn fft_relevant_res_samples_count(samples_len: usize) -> usize;
}

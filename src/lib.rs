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
//! An easy to use and fast `no_std` library (with `alloc`) to get the frequency
//! spectrum of a digital signal (e.g. audio) using FFT.
//!
//! ## Getting started
//! If you are unsure what to pick, start here. The
//! [`samples_fft_to_spectrum()`] function is the entry into the library. The
//! following configuration works for most cases: take a block of samples, apply
//! a Hann window, and divide the result by the number of samples.
//!
//! ```rust
//! use spectrum_analyzer::scaling::divide_by_N;
//! use spectrum_analyzer::windows::hann_window;
//! use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
//!
//! // your samples; the length must be a power of two
//! let samples = vec![0.0; 2048];
//!
//! let windowed = hann_window(&samples);
//! let spectrum = samples_fft_to_spectrum(
//!     &windowed,
//!     44100,
//!     FrequencyLimit::All,
//!     Some(&divide_by_N),
//! )
//! .unwrap();
//!
//! // the loudest frequency in the block
//! let (frequency, value) = spectrum.max();
//! ```
//!
//! ### How many samples?
//! More samples mean a finer frequency resolution (`sample_rate / N`), but
//! they also cover a longer time span, so the spectrum reacts more slowly to
//! changes. At 44100 Hz, 2048 samples (~46 ms, ~22 Hz per bin) are a good
//! starting point, 4096 if you need to tell close frequencies apart.
//!
//! ### What next?
//! * [`windows`]: which window function to apply
//! * [`scaling`]: which scaling to apply
//! * [`samples_fft_to_spectrum`]: what the resulting values mean
//! * [`FrequencySpectrum`]: what you can read from the result, e.g.
//!   [`FrequencySpectrum::max`] for the loudest frequency,
//!   [`FrequencySpectrum::freq_val_closest`] for one specific frequency, or
//!   [`FrequencySpectrum::data`] to iterate over all of them
//!
//! ## Examples
//! ### Scaling via dynamic closure
//! ```rust
//! use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
//! // get data from audio source, ideally in range `-1.0..=1.0`
//! let samples = vec![0.0, 1.1, 5.5, -5.5];
//! let res = samples_fft_to_spectrum(
//!         &samples,
//!         44100,
//!         FrequencyLimit::All,
//!         // Create your scaling function as closure on the fly as needed.
//!         Some(&|val, info| val - info.min),
//! );
//! ```
//! ### Scaling via static function
//! ```rust
//! use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
//! use spectrum_analyzer::scaling::divide_by_N;
//! // get data from audio source, ideally in range `-1.0..=1.0`
//! let samples = vec![0.0, 1.1, 5.5, -5.5];
//! let res = samples_fft_to_spectrum(
//!         &samples,
//!         44100,
//!         FrequencyLimit::All,
//!         // Use one of the provided scaling functions. Here, we make the
//!         // values independent of the number of samples.
//!         Some(&divide_by_N),
//! );
//! ```

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::must_use_candidate,
    clippy::undocumented_unsafe_blocks,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from,
    clippy::float_cmp
)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]
#![no_std]

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

// `vec!` is only used in tests; `alloc` itself is used throughout.
#[cfg_attr(test, macro_use)]
extern crate alloc;

pub use crate::frequency::{FiniteF32, Frequency, FrequencyValue, NonNegF32};
pub use crate::limit::FrequencyLimit;
pub use crate::limit::FrequencyLimitError;
pub use crate::spectrum::FrequencySpectrum;

use crate::error::SpectrumAnalyzerError;
use crate::fft::{Complex32, FftImpl};
use crate::scaling::SpectrumScalingFunction;
use alloc::vec::Vec;

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

/// Takes an array of samples (length must be a power of 2, such as 2048),
/// applies an FFT, and returns all frequencies with their magnitude.
///
/// If a scaling function was used, the magnitude is scaled/normalized
/// accordingly.
///
/// ## Meaning of the frequency values
/// Without a scaling function, each value is the plain magnitude of the FFT
/// result. Think of it as "how much of this frequency is in the samples",
/// but not in absolute units:
///
/// * The values grow with the number of samples: twice the samples, twice
///   the value.
/// * A window function (e.g. Hann) shrinks all values by a constant factor,
///   its coherent gain (see [`windows`]).
/// * A frequency that falls between two bins reads a bit lower than one that
///   sits exactly on a bin.
///
/// To compare spectra of different lengths, use [`scaling::divide_by_N`].
/// For the actual amplitude of a sine wave, see the details below.
///
/// ### Details
/// Each value is `sqrt(re*re + im*im)` of the corresponding FFT result,
/// optionally scaled, and relates to the input as follows:
///
/// * A sine wave with amplitude `A` on a bin frequency shows up as
///   `A * N / 2`, `N` being the number of samples. The DC (0 Hz) and Nyquist
///   bins show `A * N` instead, because they have no mirror bin.
/// * A window multiplies each sample by its coefficient before the FFT. The
///   average coefficient is the coherent gain, e.g. `0.5` for Hann, and every
///   value in the spectrum shrinks by that factor.
/// * A frequency between two bins leaks into its neighbors, so its peak reads
///   lower: up to `36%` lower without a window and `15%` with a Hann window.
///
/// So to get the amplitude of a sine wave from a one-sided spectrum: divide by
/// N and multiply by 2, because the FFT splits the sine wave's amplitude
/// between its positive- and negative-frequency bins. Do not multiply by 2 for
/// the DC and Nyquist bins, which have no separate mirror bin. Finally, divide
/// by the window's coherent gain. This gives the amplitude of the individual
/// sine wave components that make up the input signal.
///
/// ## Parameters
/// * `samples` Raw audio samples, normalized to `[-1.0; 1.0]`, which is what
///   audio APIs typically deliver. Other scales work too, as the FFT is
///   linear and the values simply scale with the input, but the normalized
///   range keeps the magnitudes small: very large samples can push a
///   magnitude out of the range of [`f32`].
///   You should apply a window function (like Hann) on the data first.
///   The final frequency resolution (spacing between two bins) is
///   `sample_rate / N`, e.g. `44100/16384 == 2.69Hz`, i.e. more samples =>
///   better accuracy/frequency resolution. The amount of samples must
///   be a power of 2. If you don't have enough data, provide zeroes.
/// * `sampling_rate` The used sampling_rate in Hertz, e.g. `44100`. It must
///   not be zero, as every frequency of the spectrum derives from it.
/// * `frequency_limit` The [`FrequencyLimit`].
/// * `scaling_fn` See [`SpectrumScalingFunction`] for details.
///
/// ## Panics
/// Everything this function can check about its input is reported as an
/// error. What is left is the magnitude of a frequency leaving the range of
/// [`f32`], which needs samples far outside the range described above: with
/// normalized samples, a magnitude never exceeds the number of samples.
///
/// ## Examples
/// ### Scaling via dynamic closure
/// ```rust
/// use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
/// // get data from audio source, ideally in range `-1.0..=1.0`
/// let samples = vec![0.0, 1.1, 5.5, -5.5];
/// let res = samples_fft_to_spectrum(
///         &samples,
///         44100,
///         FrequencyLimit::All,
///         // Create your scaling function as closure on the fly as needed.
///         Some(&|val, info| val - info.min),
/// );
/// ```
/// ### Scaling via static function
/// ```rust
/// use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
/// use spectrum_analyzer::scaling::divide_by_N;
/// // get data from audio source, ideally in range `-1.0..=1.0`
/// let samples = vec![0.0, 1.1, 5.5, -5.5];
/// let res = samples_fft_to_spectrum(
///         &samples,
///         44100,
///         FrequencyLimit::All,
///         // Use one of the provided scaling functions. Here, we make the
///         // values independent of the number of samples.
///         Some(&divide_by_N),
/// );
pub fn samples_fft_to_spectrum(
    samples: &[f32],
    sampling_rate: u32,
    frequency_limit: FrequencyLimit,
    scaling_fn: Option<&SpectrumScalingFunction>,
) -> Result<FrequencySpectrum, SpectrumAnalyzerError> {
    // do several checks on input data
    {
        if samples.len() < 2 || !samples.len().is_power_of_two() || samples.len() > 32768 {
            return Err(SpectrumAnalyzerError::InvalidLengthOfSamples);
        }
        if sampling_rate == 0 {
            return Err(SpectrumAnalyzerError::InvalidSamplingRate);
        }
        let max_detectable_frequency = sampling_rate as f32 / 2.0;

        frequency_limit
            .verify(max_detectable_frequency)
            .map_err(SpectrumAnalyzerError::InvalidFrequencyLimit)?;

        for sample in samples {
            if sample.is_nan() {
                return Err(SpectrumAnalyzerError::NaNValuesNotSupported);
            }
            if sample.is_infinite() {
                return Err(SpectrumAnalyzerError::InfinityValuesNotSupported);
            }
        }
    }

    // With FFT we transform an array of time-domain waveform samples into an
    // array of frequency-domain spectrum samples:
    // https://www.youtube.com/watch?v=z7X6jgFnB6Y

    // The FFT result has same length as input, but when we interpret the
    // result, we don't need all indices (frequency bins).

    // Applies the FFT on the samples.
    let fft_res = FftImpl::calc(samples);

    // Process FFT result into a meaningful spectrum.
    fft_result_to_spectrum(
        samples.len(),
        &fft_res,
        sampling_rate,
        frequency_limit,
        scaling_fn,
    )
}

/// Transforms the FFT result into a [`FrequencySpectrum`] by calculating the
/// corresponding frequency of each FFT result (frequency bin) and optionally
/// scales each value.
///
/// ## Parameters
/// * `samples_len` Number of input samples.
/// * `fft_result` FFT result, i.e. frequency bins.
/// * `sampling_rate` Sampling rate of the input samples, e.g. `44100 [Hz]`.
/// * `frequency_limit` Possibly the bounds of [`FrequencyLimit`] the caller is
///   interested in.
/// * `scaling_fn` Optional scaling function to modify each frequency value
///   (FFT result). See [`SpectrumScalingFunction`] for details.
#[inline]
fn fft_result_to_spectrum(
    samples_len: usize,
    fft_result: &[Complex32],
    sampling_rate: u32,
    frequency_limit: FrequencyLimit,
    scaling_fn: Option<&SpectrumScalingFunction>,
) -> Result<FrequencySpectrum, SpectrumAnalyzerError> {
    let maybe_min = frequency_limit.maybe_min();
    let maybe_max = frequency_limit.maybe_max();

    let frequency_resolution = fft_calc_frequency_resolution(sampling_rate, samples_len as u32);

    // Number of frequency bins from DC through the Nyquist frequency.
    let bin_count = samples_len / 2 + 1;
    debug_assert_eq!(fft_result.len(), bin_count);

    // Preallocate space for the maximum possible number of bins (DC component
    // up to and including the Nyquist frequency): the filtered iterator below
    // has no precise size hint, so collecting it directly would grow the
    // vector with several re-allocations.
    let mut frequency_vec = Vec::with_capacity(bin_count);

    // frequency => frequency value pairs
    let bin_iter = fft_result
        .iter()
        .enumerate()
        // Map frequency bin to corresponding frequency (Hz).
        .map(|(fr_bin, fr_val /* result of the FFT at that index */)| {
            // Let's assume we have 2048 input samples. A complex FFT produces 2048
            // complex values. For a real FFT, however, only 1024 complex values are
            // needed because the negative-frequency half is redundant.
            //
            // With a complex FFT, the relevant part of the result would be:
            //
            // N real audio samples    : [0], [1], [2], [3], ... , [2047] (N = 2048)
            // ... mapped to ...
            // N complex audio samples : [0], [1], [2], [3], ... , [2047]
            // ... put into an FFT ...
            // Relevant FFT result     : [0], [1], [2], [3], ... , [1024]
            //                            ^                            ^
            // Frequency                : 0 Hz, ..................... Sampling Rate/2
            //                            DC component                Nyquist frequency
            //                                                        (22050 Hz @ 44100 Hz)
            //
            // We use a performance-optimized real FFT with `microfft`. It performs the
            // calculation in-place: N f32 input values are transformed into N/2 complex
            // values. The first complex value is special: its real part contains the DC
            // component, while its imaginary part contains the Nyquist component.
            //
            // Thus, the 1024 complex output values contain 1025 frequency values: DC,
            // bins 1..=1023, and Nyquist. Before we called this, the FFT function
            // already unpacked the Nyquist component into an additional element of
            // the FFT result vector that we process here.

            // More information:
            // - https://stackoverflow.com/questions/4364823/ (explanation of the algorithm)
            // - https://stackoverflow.com/a/4371627/2891595
            // - https://www.gaussianwaves.com/2015/11/interpreting-fft-results-complex-dft-frequency-bins-and-fftshift/
            // - https://www.gaussianwaves.com/gaussianwaves/wp-content/uploads/2015/11/realDFT_complexDFT.png
            let fr = fr_bin as f32 * frequency_resolution;

            (fr_bin, fr, fr_val)
        })
        // Filter out frequencies we are not interested (lower threshold).
        .skip_while(|(_fr_bin, fr, _fr_val)| {
            maybe_min.is_some_and(|min_fr| {
                // Inclusive!
                // Attention: due to the frequency resolution, we do not
                // necessarily hit exactly the frequency, that a user requested
                // (e.g. 1500 Hz is requested but next matching bin is 1510 Hz).
                *fr < min_fr
            })
        })
        // Filter out frequencies we are not interested (upper threshold).
        .take_while(|(_fr_bin, fr, _fr_val)| {
            maybe_max.is_none_or(|max_fr| {
                // Inclusive!
                // Attention: due to the frequency resolution, we do not
                // necessarily hit exactly the frequency, that a user requested
                // (e.g. 1500 Hz is requested but next matching bin is 1490 Hz).
                *fr <= max_fr
            })
        })
        // FFT result is always complex: calc magnitude of complex number to get
        // the frequency value: sqrt(re*re + im*im) (re: real part, im: imaginary part)
        .map(|(_fr_bin, fr, fr_val)| {
            (
                Frequency::from(fr),
                FrequencyValue::from(complex_to_magnitude(fr_val)),
            )
        });

    // Collect all into a sorted vector (from lowest frequency to highest)
    frequency_vec.extend(bin_iter);
    // Give excess memory back if a frequency limit excluded many bins.
    frequency_vec.shrink_to_fit();

    // A valid frequency limit can still miss all FFT bins, or leave only one.
    // Statistics and interpolation require at least two frequency points.
    if frequency_vec.len() < 2 {
        return Err(SpectrumAnalyzerError::FrequencyLimitTooNarrow);
    }

    // Create the spectrum wrapper.
    let mut spectrum =
        FrequencySpectrum::new(frequency_vec, frequency_resolution, samples_len as u32);

    // Apply the scaling function.
    if let Some(scaling_fn) = scaling_fn {
        spectrum.apply_scaling_fn(scaling_fn)?
    }

    Ok(spectrum)
}

/// Calculate the frequency resolution of the FFT. It is determined by the
/// sampling rate in Hertz and N, the number of samples given into the FFT.
///
/// With the frequency resolution, we can determine the corresponding frequency
/// of each index (frequency bin) in the FFT result buffer.
///
/// ## Parameters
/// * `samples_len` Number of samples put into the FFT
/// * `sampling_rate` sampling_rate, e.g. `44100 [Hz]`
///
/// ## Return value
/// Frequency resolution in Hertz.
///
/// ## More info
/// * <https://www.researchgate.net/post/How-can-I-define-the-frequency-resolution-in-FFT-And-what-is-the-difference-on-interpreting-the-results-between-high-and-low-frequency-resolution>
/// * <https://stackoverflow.com/questions/4364823/>
#[inline]
fn fft_calc_frequency_resolution(sampling_rate: u32, samples_len: u32) -> Frequency {
    Frequency::from(sampling_rate as f32 / samples_len as f32)
}

/// Maps a [`Complex32`] to its magnitude as `f32`. This is done by calculating
/// `sqrt(re*re + im*im)`. This is required to convert the complex FFT results
/// back to real values.
///
/// ## Parameters
/// * `val` A single value from the FFT output buffer of type [`Complex32`].
///
/// ## Panics
/// If the magnitude leaves the range of [`f32`], which needs samples far
/// outside the range this library expects.
#[inline]
fn complex_to_magnitude(val: &Complex32) -> NonNegF32 {
    // calculates sqrt(re*re + im*im), i.e. magnitude of complex number
    let sum = val.re * val.re + val.im * val.im;
    NonNegF32::try_new(libm::sqrtf(sum))
        .expect("magnitude should be within the range of f32; samples are too large")
}

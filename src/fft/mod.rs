//! Abstraction over FFT implementation. This is manly necessary because we might have
//! no_std/std implementations as well as real/complex implementations.
//! This crate compiles only iff exactly one feature, i.e. one FFT implementation, is activated.

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

pub(crate) trait Fft<FftResultType> {
    /// Applies the FFT on the given implementation.
    /// If necessary, the data is converted to a complex
    /// number first. If so, the resulting vector
    /// contains the magnitudes of all complex numbers
    /// without any further normalization/scaling.
    /// The magnitude is `sqrt(re*re + im*im)`.
    ///
    /// ## Parameters
    /// * `samples` samples for FFT. Length MUST be a power of 2 for FFT, e.g. 1024 or 4096!
    ///
    /// ## Return
    /// Vector of FFT results.
    fn fft_apply(samples: &[f32]) -> Vec<FftResultType>;

    /// Maps a single result from [`fft_apply`] and maps it to `f32`.
    /// For real FFT implementations, this is equal to identity.
    /// For complex FFT implementations, this is the magnitude,
    /// e.g. `sqrt(re*re + im*im)`.
    ///
    /// ## Parameters
    /// * `val` A single value from the FFT output buffer of type [`FftResultType`].
    fn fft_map_result_to_f32(val: &FftResultType) -> f32;

    /// Calculate the frequency resolution of the FFT. It is determined by the sampling rate
    /// in Hertz and N, the number of samples given into the FFT. With the frequency resolution,
    /// we can determine the corresponding frequency of each index in the FFT result buffer.
    ///
    /// In some FFT implementations, e.g. real instead of complex, this is a little bit different.
    /// `microfft::real` slits the spectrum across the indices 0..N in output buffer  rather than
    /// 0..N/2.
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
    fn fft_calc_frequency_resolution(
        sampling_rate: u32,
        samples_len: u32,
    ) -> f32;

    /// Returns the relevant results of the FFT result. For complex numbers this is
    /// `N/2 + 1`, i.e. `0..=N/2` (inclusive end). This might be different
    /// for real FFT implementations.
    ///
    /// This function determines together with [`fft_calc_frequency_resolution`] what
    /// index in the FFT result corresponds to what frequency.
    fn fft_relevant_res_samples_count(samples_len: usize) -> usize;
}

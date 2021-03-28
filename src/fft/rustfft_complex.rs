//! Complex FFT using `rustfft`. Results should be equal to the ones from `microfft`.
//! The difference is that this implementation works only in `std`-environments
//! and can handle sample lengths of more than 4096.

use alloc::vec::Vec;

use crate::fft::Fft as FftAbstraction;
use core::convert::TryInto;
use rustfft::algorithm::Radix4;
use rustfft::{FftDirection, Fft};
use rustfft::num_complex::Complex32;

pub type FftResultType = Complex32;

/// Dummy struct with no properties but used as a type
/// to implement a concrete FFT strategy using (`rustfft::algorithm::Radix4`).
pub struct FftImpl;

impl FftImpl {
    /// Converts all samples to a complex number (imaginary part is set to zero)
    /// as preparation for the FFT.
    ///
    /// ## Parameters
    /// `samples` Input samples.
    ///
    /// ## Return value
    /// New vector with elements of FFT output/result.
    #[inline(always)]
    fn samples_to_complex(samples: &[f32]) -> Vec<Complex32> {
        samples
            .iter()
            .map(|x| Complex32::new(x.clone(), 0.0))
            .collect::<Vec<Complex32>>()
    }
}

impl FftAbstraction<FftResultType> for FftImpl {

    #[inline(always)]
    fn fft_apply(samples: &[f32]) -> Vec<FftResultType> {
        let mut samples = Self::samples_to_complex(samples);
        let fft = Radix4::new(samples.len(), FftDirection::Forward);
        fft.process(&mut samples);
        samples
    }

    #[inline(always)]
    fn fft_map_result_to_f32(val: &FftResultType) -> f32 {
        // calculates sqrt(re*re + im*im), i.e. magnitude of complex number
        let sum = val.re * val.re + val.im * val.im;
        let sqrt = sum.sqrt();
        debug_assert!(sqrt != f32::NAN, "sqrt is NaN!");
        sqrt
    }

    #[inline(always)]
    fn fft_calc_frequency_resolution(sampling_rate: u32, samples_len: u32) -> f32 {
        sampling_rate as f32 / samples_len as f32
    }

    #[inline(always)]
    fn fft_relevant_res_samples_count(samples_len: usize) -> usize {
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
        samples_len / 2 + 1
    }
}

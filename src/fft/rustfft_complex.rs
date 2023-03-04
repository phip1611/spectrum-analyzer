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
//! Complex FFT using `rustfft`. Results should be equal to the ones from `microfft`.
//! The difference is that this implementation works only in `std`-environments
//! and can handle sample lengths of more than 4096.

use alloc::vec::Vec;

use crate::fft::Fft as FftAbstraction;
use rustfft::algorithm::Radix4;
use rustfft::{Fft, FftDirection};

/// The result of a FFT is always complex but because different FFT crates might
/// use different versions of "num-complex", each implementation exports
/// it's own version that gets used in lib.rs for binary compatibility.
pub use rustfft::num_complex::Complex32;

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
    #[inline]
    fn samples_to_complex(samples: &[f32]) -> Vec<Complex32> {
        samples
            .iter()
            .map(|x| Complex32::new(*x, 0.0))
            .collect::<Vec<Complex32>>()
    }
}

impl FftAbstraction<Complex32> for FftImpl {
    #[inline]
    fn fft_apply(samples: &[f32]) -> Vec<Complex32> {
        let mut samples = Self::samples_to_complex(samples);
        let fft = Radix4::new(samples.len(), FftDirection::Forward);
        fft.process(&mut samples);
        samples
    }

    #[inline]
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

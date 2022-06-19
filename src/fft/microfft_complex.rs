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
//! Complex FFT using `microfft::complex`. Results should be equal to the ones from `rustfft`.
//! The difference is that this implementation works in `no_std`-environments but it is
//! limited to a maximum sample length of 16384 (with microfft version 0.5.0)

use alloc::vec::Vec;

use crate::fft::Fft;
use core::convert::TryInto;
use microfft::complex;

/// The result of a FFT is always complex but because different FFT crates might
/// use different versions of "num-complex", each implementation exports
/// it's own version that gets used in lib.rs for binary compatibility.
pub use microfft::Complex32;

/// Dummy struct with no properties but used as a type
/// to implement a concrete FFT strategy using (`microfft::complex`).
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
            .map(|x| Complex32::new(*x, 0.0))
            .collect::<Vec<Complex32>>()
    }
}

impl Fft<Complex32> for FftImpl {
    #[inline(always)]
    fn fft_apply(samples: &[f32]) -> Vec<Complex32> {
        let buffer = Self::samples_to_complex(samples);

        if buffer.len() == 2 {
            let mut buffer: [_; 2] = buffer.try_into().unwrap();
            complex::cfft_2(&mut buffer).to_vec()
        } else if buffer.len() == 4 {
            let mut buffer: [_; 4] = buffer.try_into().unwrap();
            complex::cfft_4(&mut buffer).to_vec()
        } else if buffer.len() == 8 {
            let mut buffer: [_; 8] = buffer.try_into().unwrap();
            complex::cfft_8(&mut buffer).to_vec()
        } else if buffer.len() == 16 {
            let mut buffer: [_; 16] = buffer.try_into().unwrap();
            complex::cfft_16(&mut buffer).to_vec()
        } else if buffer.len() == 32 {
            let mut buffer: [_; 32] = buffer.try_into().unwrap();
            complex::cfft_32(&mut buffer).to_vec()
        } else if buffer.len() == 64 {
            let mut buffer: [_; 64] = buffer.try_into().unwrap();
            complex::cfft_64(&mut buffer).to_vec()
        } else if buffer.len() == 128 {
            let mut buffer: [_; 128] = buffer.try_into().unwrap();
            complex::cfft_128(&mut buffer).to_vec()
        } else if buffer.len() == 256 {
            let mut buffer: [_; 256] = buffer.try_into().unwrap();
            complex::cfft_256(&mut buffer).to_vec()
        } else if buffer.len() == 512 {
            let mut buffer: [_; 512] = buffer.try_into().unwrap();
            complex::cfft_512(&mut buffer).to_vec()
        } else if buffer.len() == 1024 {
            let mut buffer: [_; 1024] = buffer.try_into().unwrap();
            complex::cfft_1024(&mut buffer).to_vec()
        } else if buffer.len() == 2048 {
            let mut buffer: [_; 2048] = buffer.try_into().unwrap();
            complex::cfft_2048(&mut buffer).to_vec()
        } else if buffer.len() == 4096 {
            let mut buffer: [_; 4096] = buffer.try_into().unwrap();
            complex::cfft_4096(&mut buffer).to_vec()
        } else {
            panic!("`microfft::complex` only supports powers of 2 between 2 and 4096!");
        }
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

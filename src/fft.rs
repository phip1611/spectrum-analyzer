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

//! Real FFT using [`microfft::real`] that is very fast and also works in
//! `no_std` environments. It is faster than regular fft (with the `rustfft`
//! crate for example). The difference to a complex FFT, as with `rustfft` is,
//! that the result vector contains fewer results as there are no mirrored
//! frequencies.

/// FFT base result type.
pub use microfft::Complex32;

use alloc::vec::Vec;
use core::convert::TryInto;
use core::mem;
use microfft::real;

/// Calculates the FFT by invoking the function of [`microfft::real`] that
/// corresponds to the input size.
macro_rules! real_fft_n {
    ($buffer:expr, $( $i:literal ),*) => {
        match $buffer.len() {
            $(
                $i => {
                    let fixed_size_view = $buffer.as_mut_slice().try_into().unwrap();
                    paste::paste! (
                        real::[<rfft_$i>]
                    )(fixed_size_view)
                }
            )*
            _ => { unimplemented!("should be one of the supported buffer lengths, but was {}", $buffer.len()) }
        }
    };
}

/// FFT using [`microfft::real`].
pub struct FftImpl;

impl FftImpl {
    /// Calculates the FFT For the given input samples and returns a [`Vec`] of
    /// [`Complex32`] with length `samples.len() / 2 + 1`.
    ///
    /// The first index corresponds to the DC component and the last index to
    /// the Nyquist frequency.
    ///
    /// # Parameters
    /// - `samples`: Array with samples. Each value must be a regular floating
    ///              point number (no NaN or infinite) and the length must be
    ///              a power of two. Otherwise, the function panics.
    #[inline]
    pub(crate) fn calc(samples: &[f32]) -> Vec<Complex32> {
        assert_eq!(
            samples.len() % 2,
            0,
            "buffer length must be a multiple of two!"
        );
        let mut vec_buffer = Vec::with_capacity(samples.len() + 2 /* Nyquist */);
        assert_eq!(
            vec_buffer.capacity() % 2,
            0,
            "vector capacity must be a multiple of two for safe casting!"
        );

        vec_buffer.extend_from_slice(samples);

        // The result is a view into the buffer.
        // We discard the view and directly operate on the buffer.
        let _fft_res: &mut [Complex32] = real_fft_n!(
            &mut vec_buffer,
            2,
            4,
            8,
            16,
            32,
            64,
            128,
            256,
            512,
            1024,
            2048,
            4096,
            8192,
            16384,
            32768
        );

        // We transform the original vector while preserving its memory, to
        // prevent any reallocation or unnecessary copying.
        let mut buffer = {
            let ptr = vec_buffer.as_mut_ptr().cast::<Complex32>();
            let len = vec_buffer.len() / 2;
            let capacity = vec_buffer.capacity() / 2;
            let new_buffer_view = unsafe { Vec::from_raw_parts(ptr, len, capacity) };
            mem::forget(vec_buffer);
            new_buffer_view
        };

        // `microfft::real` documentation says: the Nyquist frequency real value
        // is packed inside the imaginary part of the DC component.
        let nyquist_fr_pos_val = buffer[0].im;
        buffer[0].im = 0.0;
        // manually add the Nyquist frequency
        buffer.push(Complex32::new(nyquist_fr_pos_val, 0.0));
        buffer
    }
}

#[cfg(test)]
mod tests {
    use crate::fft::FftImpl;

    /// This test is primarily for miri.
    #[test]
    fn test_memory_safety() {
        let samples = [1.0, 2.0, 3.0, 4.0];
        let fft = FftImpl::calc(&samples);

        assert_eq!(fft.len(), 2 + 1);
    }
}

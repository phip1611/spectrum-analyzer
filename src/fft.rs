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

//! Real FFT using [`microfft::real`] that is very fast and also works in `no_std`
//! environments. It is faster than regular fft (with the `rustfft` crate for
//! example). The difference to a complex FFT, as with `rustfft` is, that the
//! result vector contains less results as there are no mirrored frequencies.

use alloc::vec::Vec;
use core::convert::TryInto;
use microfft::real;

/// The result of a FFT is always complex but because different FFT crates might
/// use different versions of "num-complex", each implementation exports
/// it's own version that gets used in lib.rs for binary compatibility.
pub use microfft::Complex32;

/// Calculates the real FFT by invoking the proper function corresponding to the
/// buffer length.
macro_rules! real_fft_n {
    ($buffer:expr, $( $i:literal ),*) => {
        match $buffer.len() {
            $(
                $i => {
                    let mut buffer: [_; $i] = $buffer.try_into().unwrap();
                    paste::paste! (
                        real::[<rfft_$i>]
                    )(&mut buffer).to_vec()
                }
            )*
            _ => { unimplemented!("unexpected buffer len") }
        }
    };
}

/// Real FFT using [`microfft::real`].
pub struct FftImpl;

impl FftImpl {
    /// Calculates the FFT For the given input samples and returns a Vector of
    /// of [`Complex32`] with length `samples.len() / 2 + 1`, where the first
    /// index corresponds to the DC component and the last index to the Nyquist
    /// frequency.
    ///
    /// # Parameters
    /// - `samples`: Array with samples. Each value must be a regular floating
    ///              point number (no NaN or infinite) and the length must be
    ///              a power of two. Otherwise, the function panics.
    #[inline]
    pub(crate) fn calc(samples: &[f32]) -> Vec<Complex32> {
        let mut fft_res: Vec<Complex32> =
            real_fft_n!(samples, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384);

        // `microfft::real` documentation says: the Nyquist frequency real value
        // is packed inside the imaginary part of the DC component.
        let nyquist_fr_pos_val = fft_res[0].im;
        fft_res[0].im = 0.0;
        // manually add the nyquist frequency
        fft_res.push(Complex32::new(nyquist_fr_pos_val, 0.0));
        fft_res
    }
}

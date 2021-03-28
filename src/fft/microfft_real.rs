//! Abstraction over FFT implementation (in future maybe dependent by Cargo features).
//! This compiles only iff exactly one feature, i.e. one FFT implementation, is activated.

use alloc::vec::Vec;

use microfft::real;
use crate::fft::Fft;
use core::convert::TryInto;

pub type FftResultType = f32;

/// Dummy struct with no properties but used as a type
/// to implement a concrete FFT strategy using (`microfft::real`).
pub struct FftImpl;


impl Fft<FftResultType> for FftImpl {

    #[inline(always)]
    fn fft_apply(samples: &[f32]) -> Vec<FftResultType> {
        let buffer = samples;
        if buffer.len() == 2 {
            let mut buffer: [_; 2] = buffer.try_into().unwrap();
            real::rfft_2(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 4 {
            let mut buffer: [_; 4] = buffer.try_into().unwrap();
            real::rfft_4(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 8 {
            let mut buffer: [_; 8] = buffer.try_into().unwrap();
            real::rfft_8(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 16 {
            let mut buffer: [_; 16] = buffer.try_into().unwrap();
            real::rfft_16(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 32 {
            let mut buffer: [_; 32] = buffer.try_into().unwrap();
            real::rfft_32(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 64 {
            let mut buffer: [_; 64] = buffer.try_into().unwrap();
            real::rfft_64(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 128 {
            let mut buffer: [_; 128] = buffer.try_into().unwrap();
            real::rfft_128(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 256 {
            let mut buffer: [_; 256] = buffer.try_into().unwrap();
            real::rfft_256(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 512 {
            let mut buffer: [_; 512] = buffer.try_into().unwrap();
            real::rfft_512(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 1024 {
            let mut buffer: [_; 1024] = buffer.try_into().unwrap();
            real::rfft_1024(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 2048 {
            let mut buffer: [_; 2048] = buffer.try_into().unwrap();
            real::rfft_2048(&mut buffer);
            buffer.to_vec()
        } else if buffer.len() == 4096 {
            let mut buffer: [_; 4096] = buffer.try_into().unwrap();
            real::rfft_4096(&mut buffer);
            buffer.to_vec()
        } else {
            panic!("`microfft::complex` only supports powers of 2 between and 4096!");
        }
    }

    #[inline(always)]
    fn fft_map_result_to_f32(val: &FftResultType) -> f32 {
        // original value, identity
        *val
    }

    #[inline(always)]
    fn fft_calc_frequency_resolution(sampling_rate: u32, samples_len: u32) -> f32 {
        sampling_rate as f32 / samples_len as f32 / 2.0
    }
}

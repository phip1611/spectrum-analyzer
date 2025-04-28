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
//! Errors related to the spectrum analysis via FFT. Most probably, the errors will
//! result in wrong input data, before the actual calculation has begun.
//!
//! This module focuses on the "overall" errors. More specific errors might be
//! located in submodules.

use crate::limit::FrequencyLimitError;
use core::error::Error;
use core::fmt::{Display, Formatter};

/// Describes main errors of the library. Almost all errors
/// are caused by wrong input.
#[derive(Debug)]
pub enum SpectrumAnalyzerError {
    /// There must be at least two samples.
    TooFewSamples,
    /// NaN values in samples are not supported!
    NaNValuesNotSupported,
    /// Infinity-values (regarding floating point representation) in samples are not supported!
    InfinityValuesNotSupported,
    /// The frequency is invalid. See [`FrequencyLimitError`].
    InvalidFrequencyLimit(FrequencyLimitError),
    /// The number of samples must be a power of two in order for the FFT.
    SamplesLengthNotAPowerOfTwo,
    /// After applying the scaling function on a specific item, the returned value is either
    /// infinity or NaN, according to IEEE-754. This is invalid. Check
    /// your scaling function!
    ScalingError(f32, f32),
}

impl Display for SpectrumAnalyzerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SpectrumAnalyzerError::TooFewSamples => write!(f, "Too few samples!"),
            SpectrumAnalyzerError::NaNValuesNotSupported => {
                write!(f, "NaN values are not supported!")
            }
            SpectrumAnalyzerError::InfinityValuesNotSupported => {
                write!(f, "Infinity values are not supported!")
            }
            SpectrumAnalyzerError::InvalidFrequencyLimit(e) => {
                write!(f, "Invalid frequency limit: {}", e)
            }
            SpectrumAnalyzerError::SamplesLengthNotAPowerOfTwo => {
                write!(f, "Samples length must be a power of two!")
            }
            SpectrumAnalyzerError::ScalingError(a, b) => write!(f, "Scaling error: {} -> {}", a, b),
        }
    }
}

impl Error for SpectrumAnalyzerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SpectrumAnalyzerError::InvalidFrequencyLimit(e) => Some(e),
            _ => None,
        }
    }
}

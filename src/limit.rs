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
//! Module for the struct [`FrequencyLimit`].

/// Can be used to specify a desired frequency limit. If you know that you only
/// need frequencies `f <= 1000Hz`, `1000 <= f <= 6777`, or `10000 <= f`, then this
/// can help you to accelerate overall computation speed and memory usage.
///
/// Please note that due to frequency inaccuracies the FFT result may not contain
/// a value for `1000Hz` but for `998.76Hz`!
#[derive(Debug, Copy, Clone)]
pub enum FrequencyLimit {
    /// Interested in all frequencies. [0, sampling_rate/2] (Nyquist theorem).
    All,
    /// Only interested in frequencies `f <= 1000Hz` for example. Limit is inclusive.
    Min(f32),
    /// Only interested in frequencies `10000 <= f` for example. Limit is inclusive.
    Max(f32),
    /// Only interested in frequencies `1000 <= f <= 6777` for example. Both values are inclusive.
    Range(f32, f32),
}

impl FrequencyLimit {
    #[inline(always)]
    pub fn maybe_min(&self) -> Option<f32> {
        match self {
            FrequencyLimit::Min(min) => Some(*min),
            FrequencyLimit::Range(min, _) => Some(*min),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn maybe_max(&self) -> Option<f32> {
        match self {
            FrequencyLimit::Max(max) => Some(*max),
            FrequencyLimit::Range(_, max) => Some(*max),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn min(&self) -> f32 {
        self.maybe_min().expect("Must contain a value!")
    }

    #[inline(always)]
    pub fn max(&self) -> f32 {
        self.maybe_max().expect("Must contain a value!")
    }
}

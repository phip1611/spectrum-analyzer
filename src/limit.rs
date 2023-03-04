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
    /// Semantically equivalent to "None" limit at all).
    All,
    /// Only interested in frequencies `Frequency <= x`. Limit is inclusive.
    /// Supported values are `0 <= x <= Nyquist-Frequency`.
    Min(f32),
    /// Only interested in frequencies `x <= Frequency`. Limit is inclusive.
    /// Supported values are `0 <= x <= N`.
    Max(f32),
    /// Only interested in frequencies `1000 <= f <= 6777` for example. Both values are inclusive.
    /// The first value of the tuple is equivalent to [`FrequencyLimit::Min`] and the latter
    /// equivalent to [`FrequencyLimit::Max`]. Furthermore, the first value must not be
    /// bigger than the second value.
    Range(f32, f32),
}

impl FrequencyLimit {
    /// Returns the minimum value, if any.
    #[inline]
    #[must_use]
    pub const fn maybe_min(&self) -> Option<f32> {
        match self {
            Self::Min(min) => Some(*min),
            Self::Range(min, _) => Some(*min),
            _ => None,
        }
    }

    /// Returns the maximum value, if any.
    #[inline]
    #[must_use]
    pub const fn maybe_max(&self) -> Option<f32> {
        match self {
            Self::Max(max) => Some(*max),
            Self::Range(_, max) => Some(*max),
            _ => None,
        }
    }

    /// Returns the minimum value, panics if it's none.
    /// Unwrapped version of [`Self::maybe_min`].
    #[inline]
    #[must_use]
    pub fn min(&self) -> f32 {
        self.maybe_min().expect("Must contain a value!")
    }

    /// Returns the minimum value, panics if it's none.
    /// Unwrapped version of [`Self::maybe_max`].
    #[inline]
    #[must_use]
    pub fn max(&self) -> f32 {
        self.maybe_max().expect("Must contain a value!")
    }

    /// Verifies that the frequency limit has sane values and takes the maximum possible
    /// frequency into account.
    pub fn verify(&self, max_detectable_frequency: f32) -> Result<(), FrequencyLimitError> {
        match self {
            Self::All => Ok(()),
            Self::Min(x) | Self::Max(x) => {
                if *x < 0.0 {
                    Err(FrequencyLimitError::ValueBelowMinimum(*x))
                } else if *x > max_detectable_frequency {
                    Err(FrequencyLimitError::ValueAboveNyquist(*x))
                } else {
                    Ok(())
                }
            }
            Self::Range(min, max) => {
                Self::Min(*min).verify(max_detectable_frequency)?;
                Self::Max(*max).verify(max_detectable_frequency)?;
                if min > max {
                    Err(FrequencyLimitError::InvalidRange(*min, *max))
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Possible errors when creating a [`FrequencyLimit`]-object.
#[derive(Debug)]
pub enum FrequencyLimitError {
    /// If the minimum value is below 0. Negative frequencies are not supported.
    ValueBelowMinimum(f32),
    /// If the maximum value is above Nyquist frequency. Nyquist-Frequency is the maximum
    /// detectable frequency.
    ValueAboveNyquist(f32),
    /// Either the corresponding value is below or above the minimum/maximum or the
    /// first member of the tuple is bigger than the second.
    InvalidRange(f32, f32),
}

#[cfg(test)]
mod tests {
    use crate::FrequencyLimit;

    #[test]
    fn test_panic_min_below_minimum() {
        let _ = FrequencyLimit::Min(-1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_panic_min_above_nyquist() {
        let _ = FrequencyLimit::Min(1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_panic_max_below_minimum() {
        let _ = FrequencyLimit::Max(-1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_panic_max_above_nyquist() {
        let _ = FrequencyLimit::Max(1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_panic_range_min() {
        let _ = FrequencyLimit::Range(-1.0, 0.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_panic_range_max() {
        let _ = FrequencyLimit::Range(0.0, 1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_panic_range_order() {
        let _ = FrequencyLimit::Range(0.0, -1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_ok() {
        FrequencyLimit::Min(50.0).verify(100.0).unwrap();
        FrequencyLimit::Max(50.0).verify(100.0).unwrap();
        // useless, but not an hard error
        FrequencyLimit::Range(50.0, 50.0).verify(100.0).unwrap();
        FrequencyLimit::Range(50.0, 70.0).verify(100.0).unwrap();
    }
}

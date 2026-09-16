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

use crate::NonNegF32;
use core::error::Error;
use core::fmt::{Display, Formatter};

/// Can be used to specify a desired frequency limit.
///
/// If you know that you only need frequencies `f <= 1000Hz`,
/// `1000 <= f <= 6777`, or `10000 <= f`, then this can help you to accelerate
/// overall computation speed and memory usage.
///
/// Please note that due to frequency inaccuracies the FFT result may not contain
/// a value for `1000Hz` but for `998.76Hz`!
#[derive(Debug, Copy, Clone)]
pub enum FrequencyLimit {
    /// Interested in all frequencies, including the DC component up to the
    /// Nyquist frequency. In other words, no limit at all.
    All,
    /// Lower bound: only interested in frequencies `>= x`. Limit is
    /// inclusive. Supported values are `0 <= x <= Nyquist-Frequency`.
    Min(NonNegF32),
    /// Upper bound: only interested in frequencies `<= x`. Limit is
    /// inclusive. Supported values are `0 <= x <= Nyquist-Frequency`.
    Max(NonNegF32),
    /// Only interested in frequencies `1000 <= f <= 6777` for example. Both values are inclusive.
    /// The first value of the tuple is equivalent to [`FrequencyLimit::Min`] and the latter
    /// equivalent to [`FrequencyLimit::Max`]. Furthermore, the first value must not be
    /// bigger than the second value.
    Range(NonNegF32, NonNegF32),
}

impl FrequencyLimit {
    /// Creates a [`Self::Min`] limit.
    ///
    /// # Panics
    /// If `min` is negative or not finite.
    #[inline]
    #[must_use]
    pub fn min(min: impl Into<NonNegF32>) -> Self {
        Self::Min(min.into())
    }

    /// Creates a [`Self::Max`] limit.
    ///
    /// # Panics
    /// If `max` is negative or not finite.
    #[inline]
    #[must_use]
    pub fn max(max: impl Into<NonNegF32>) -> Self {
        Self::Max(max.into())
    }

    /// Creates a [`Self::Range`] limit.
    ///
    /// # Panics
    /// If one of the values is negative or not finite or if `min > max`.
    #[inline]
    #[must_use]
    pub fn range(min: impl Into<NonNegF32>, max: impl Into<NonNegF32>) -> Self {
        let min = min.into();
        let max = max.into();
        assert!(min <= max, "min should not be bigger than max");
        Self::Range(min, max)
    }

    /// Returns the minimum value, if any.
    #[inline]
    #[must_use]
    pub const fn maybe_min(&self) -> Option<NonNegF32> {
        match self {
            Self::Min(min) => Some(*min),
            Self::Range(min, _) => Some(*min),
            _ => None,
        }
    }

    /// Returns the maximum value, if any.
    #[inline]
    #[must_use]
    pub const fn maybe_max(&self) -> Option<NonNegF32> {
        match self {
            Self::Max(max) => Some(*max),
            Self::Range(_, max) => Some(*max),
            _ => None,
        }
    }

    /// Verifies that the frequency limit has sane values and takes the maximum possible
    /// frequency into account.
    pub fn verify(&self, max_detectable_frequency: f32) -> Result<(), FrequencyLimitError> {
        match self {
            Self::All => Ok(()),
            Self::Min(x) | Self::Max(x) => {
                if *x > max_detectable_frequency {
                    Err(FrequencyLimitError::ValueAboveNyquist(x.val()))
                } else {
                    Ok(())
                }
            }
            Self::Range(min, max) => {
                Self::Min(*min).verify(max_detectable_frequency)?;
                Self::Max(*max).verify(max_detectable_frequency)?;
                if min > max {
                    Err(FrequencyLimitError::InvalidRange(min.val(), max.val()))
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
    /// If the maximum value is above Nyquist frequency. Nyquist-Frequency is the maximum
    /// detectable frequency.
    ValueAboveNyquist(f32),
    /// The first member of the tuple is bigger than the second. A value above
    /// the Nyquist frequency is reported as [`Self::ValueAboveNyquist`], even
    /// inside a range.
    InvalidRange(f32, f32),
}

impl Display for FrequencyLimitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ValueAboveNyquist(x) => write!(f, "Value above Nyquist: {x}"),
            Self::InvalidRange(min, max) => write!(f, "Invalid range: {min} <= x <= {max}"),
        }
    }
}

impl Error for FrequencyLimitError {}

#[cfg(test)]
mod tests {
    use crate::limit::FrequencyLimitError;
    use crate::{FrequencyLimit, NonNegF32};

    #[test]
    #[should_panic(expected = "value should be finite and not negative")]
    fn test_construction_rejects_not_a_number() {
        let _ = FrequencyLimit::min(f32::NAN);
    }

    #[test]
    #[should_panic(expected = "value should be finite and not negative")]
    fn test_construction_rejects_negative() {
        let _ = FrequencyLimit::max(-1.0);
    }

    #[test]
    fn test_min_above_nyquist() {
        let _ = FrequencyLimit::min(1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_max_above_nyquist() {
        let _ = FrequencyLimit::max(1.0).verify(0.0).unwrap_err();
    }

    #[test]
    fn test_range_above_nyquist() {
        let _ = FrequencyLimit::range(0.0, 1.0).verify(0.0).unwrap_err();
    }

    #[test]
    #[should_panic(expected = "min should not be bigger than max")]
    fn test_range_rejects_wrong_order() {
        let _ = FrequencyLimit::range(1.0, 0.0);
    }

    #[test]
    fn test_range_allows_equal_bounds() {
        let limit = FrequencyLimit::range(50.0, 50.0);

        assert_eq!(50.0, limit.maybe_min().unwrap());
        assert_eq!(50.0, limit.maybe_max().unwrap());
    }

    /// The constructor rejects a wrong order, but the variant itself is
    /// public, so `verify()` still has to.
    #[test]
    fn test_verify_catches_a_wrong_range() {
        let limit = FrequencyLimit::Range(NonNegF32::from(1.0), NonNegF32::from(0.0));

        assert!(matches!(
            limit.verify(1.0),
            Err(FrequencyLimitError::InvalidRange(_, _))
        ));
    }

    #[test]
    fn test_constructors_fill_the_right_bound() {
        let min = FrequencyLimit::min(50.0);
        assert_eq!(50.0, min.maybe_min().unwrap());
        assert_eq!(None, min.maybe_max());

        let max = FrequencyLimit::max(70.0);
        assert_eq!(None, max.maybe_min());
        assert_eq!(70.0, max.maybe_max().unwrap());

        let range = FrequencyLimit::range(50.0, 70.0);
        assert_eq!(50.0, range.maybe_min().unwrap());
        assert_eq!(70.0, range.maybe_max().unwrap());

        assert_eq!(None, FrequencyLimit::All.maybe_min());
        assert_eq!(None, FrequencyLimit::All.maybe_max());
    }

    #[test]
    fn test_ok() {
        FrequencyLimit::min(50.0).verify(100.0).unwrap();
        FrequencyLimit::max(50.0).verify(100.0).unwrap();
        // useless, but not an hard error
        FrequencyLimit::range(50.0, 50.0).verify(100.0).unwrap();
        FrequencyLimit::range(50.0, 70.0).verify(100.0).unwrap();
    }
}

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
//! Module for [`FiniteF32`] and [`NonNegF32`], and the two convenient type
//! definitions [`Frequency`] and [`FrequencyValue`] built on them.

use core::cmp::Ordering;
use core::fmt::{Debug, Display, Formatter, Result};
use core::ops::{Add, Div, Mul, Neg, Sub};

/// A frequency in Hertz, which is never negative.
pub type Frequency = NonNegF32;
/// The value of a [`Frequency`] in a frequency spectrum.
///
/// It is the magnitude of the FFT result at that frequency, optionally
/// scaled. A scaling function can make it negative, for example
/// [`crate::scaling::scale_20_times_log10`].
///
/// See [`crate::samples_fft_to_spectrum`] for what this means in practice.
pub type FrequencyValue = FiniteF32;

/// Wrapper around [`f32`] that guarantees a finite number, i.e., neither `NaN`
/// nor infinite. This makes the number orderable and sortable.
///
/// The type compares and calculates with [`f32`] directly, so there is rarely
/// a need to unwrap it:
///
/// ```
/// use spectrum_analyzer::FiniteF32;
///
/// let value = FiniteF32::from(0.5);
/// assert!(value > 0.25);
/// assert_eq!(value, 0.5);
/// assert_eq!(value * 2.0, 1.0);
/// ```
///
/// # Panics
/// Creating a value from an [`f32`] that is not finite panics, and so does an
/// operation between two values of this type whose result is not finite.
/// [`Self::try_new`] checks instead.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct FiniteF32(f32);

impl FiniteF32 {
    /// Creates a new value, or `None` if `val` is not finite.
    #[inline]
    #[must_use]
    pub const fn try_new(val: f32) -> Option<Self> {
        if val.is_finite() {
            Some(Self(val))
        } else {
            None
        }
    }

    /// Returns the underlying [`f32`].
    #[inline]
    #[must_use]
    pub const fn val(self) -> f32 {
        self.0
    }
}

impl From<f32> for FiniteF32 {
    /// # Panics
    /// If `val` is `NaN` or infinite.
    #[inline]
    fn from(val: f32) -> Self {
        Self::try_new(val).expect("value should be finite")
    }
}

impl From<FiniteF32> for f32 {
    #[inline]
    fn from(val: FiniteF32) -> Self {
        val.0
    }
}

impl Display for FiniteF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for FiniteF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.0)
    }
}

impl Ord for FiniteF32 {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 == other.0 {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl Eq for FiniteF32 {}

impl PartialEq for FiniteF32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for FiniteF32 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<f32> for FiniteF32 {
    #[inline]
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<FiniteF32> for f32 {
    #[inline]
    fn eq(&self, other: &FiniteF32) -> bool {
        *self == other.0
    }
}

impl PartialOrd<f32> for FiniteF32 {
    #[inline]
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<FiniteF32> for f32 {
    #[inline]
    fn partial_cmp(&self, other: &FiniteF32) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Neg for FiniteF32 {
    type Output = Self;

    /// Negating a finite number always yields a finite number.
    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

/// Implements an operator for [`FiniteF32`]. Between two of them the guarantee
/// holds, so the result is wrapped again; with a plain [`f32`] the guarantee is
/// gone and the result is an [`f32`].
macro_rules! impl_op {
    ($trait:ident, $method:ident) => {
        impl $trait for FiniteF32 {
            type Output = Self;

            /// # Panics
            /// If the result is not finite.
            #[inline]
            fn $method(self, rhs: Self) -> Self::Output {
                Self::from($trait::$method(self.0, rhs.0))
            }
        }

        impl $trait<f32> for FiniteF32 {
            type Output = f32;

            #[inline]
            fn $method(self, rhs: f32) -> Self::Output {
                $trait::$method(self.0, rhs)
            }
        }

        impl $trait<FiniteF32> for f32 {
            type Output = Self;

            #[inline]
            fn $method(self, rhs: FiniteF32) -> Self::Output {
                $trait::$method(self, rhs.0)
            }
        }
    };
}

impl_op!(Add, add);
impl_op!(Sub, sub);
impl_op!(Mul, mul);
impl_op!(Div, div);

/// Wrapper around [`FiniteF32`] that additionally guarantees a number that is
/// not negative, so `0.0` or higher.
///
/// Like [`FiniteF32`], it compares and calculates with [`f32`] directly.
/// Operations that can leave the range, such as a subtraction or a negation,
/// return a [`FiniteF32`].
///
/// ```
/// use spectrum_analyzer::NonNegF32;
///
/// let value = NonNegF32::from(0.5);
/// assert_eq!(value, 0.5);
/// assert!(value > 0.25);
/// assert_eq!(NonNegF32::from(0.25) - value, -0.25);
/// ```
///
/// # Panics
/// Creating a value from a number that is negative or not finite panics, and
/// so does an operation whose result leaves the range.
/// [`Self::try_new`] checks instead.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct NonNegF32(FiniteF32);

impl NonNegF32 {
    /// Creates a new value, or `None` if `val` is negative or not finite.
    #[inline]
    #[must_use]
    pub const fn try_new(val: f32) -> Option<Self> {
        match FiniteF32::try_new(val) {
            Some(val) if val.val() >= 0.0 => Some(Self(val)),
            _ => None,
        }
    }

    /// Returns the underlying [`f32`].
    #[inline]
    #[must_use]
    pub const fn val(self) -> f32 {
        self.0.val()
    }
}

impl From<f32> for NonNegF32 {
    /// # Panics
    /// If `val` is negative, `NaN` or infinite.
    #[inline]
    fn from(val: f32) -> Self {
        Self::try_new(val).expect("value should be finite and not negative")
    }
}

impl From<FiniteF32> for NonNegF32 {
    /// # Panics
    /// If `val` is negative.
    #[inline]
    fn from(val: FiniteF32) -> Self {
        Self::try_new(val.val()).expect("value should not be negative")
    }
}

impl From<NonNegF32> for FiniteF32 {
    #[inline]
    fn from(val: NonNegF32) -> Self {
        val.0
    }
}

impl From<NonNegF32> for f32 {
    #[inline]
    fn from(val: NonNegF32) -> Self {
        val.val()
    }
}

impl Display for NonNegF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for NonNegF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&self.0, f)
    }
}

impl Ord for NonNegF32 {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Eq for NonNegF32 {}

impl PartialEq for NonNegF32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for NonNegF32 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<f32> for NonNegF32 {
    #[inline]
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<NonNegF32> for f32 {
    #[inline]
    fn eq(&self, other: &NonNegF32) -> bool {
        *self == other.0
    }
}

impl PartialOrd<f32> for NonNegF32 {
    #[inline]
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<NonNegF32> for f32 {
    #[inline]
    fn partial_cmp(&self, other: &NonNegF32) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Neg for NonNegF32 {
    type Output = FiniteF32;

    /// Negating leaves the range, so the result is a [`FiniteF32`].
    #[inline]
    fn neg(self) -> Self::Output {
        -self.0
    }
}

impl Sub for NonNegF32 {
    type Output = FiniteF32;

    /// A subtraction can leave the range, so the result is a [`FiniteF32`].
    /// The difference of two finite numbers of the same sign is finite, so
    /// this cannot panic.
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        FiniteF32::from(self.val() - rhs.val())
    }
}

/// Implements an operator that cannot leave the range of [`NonNegF32`].
macro_rules! impl_non_neg_op {
    ($trait:ident, $method:ident) => {
        impl $trait for NonNegF32 {
            type Output = Self;

            /// # Panics
            /// If the result is not finite.
            #[inline]
            fn $method(self, rhs: Self) -> Self::Output {
                Self::from($trait::$method(self.val(), rhs.val()))
            }
        }

        impl $trait<f32> for NonNegF32 {
            type Output = f32;

            #[inline]
            fn $method(self, rhs: f32) -> Self::Output {
                $trait::$method(self.val(), rhs)
            }
        }

        impl $trait<NonNegF32> for f32 {
            type Output = Self;

            #[inline]
            fn $method(self, rhs: NonNegF32) -> Self::Output {
                $trait::$method(self, rhs.val())
            }
        }
    };
}

impl_non_neg_op!(Add, add);
impl_non_neg_op!(Mul, mul);
impl_non_neg_op!(Div, div);

impl Sub<f32> for NonNegF32 {
    type Output = f32;

    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        self.val() - rhs
    }
}

impl Sub<NonNegF32> for f32 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: NonNegF32) -> Self::Output {
        self - rhs.val()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_f32_construction() {
        assert_eq!(Some(FiniteF32(0.5)), FiniteF32::try_new(0.5));
        assert_eq!(None, FiniteF32::try_new(f32::NAN));
        assert_eq!(None, FiniteF32::try_new(f32::INFINITY));
        assert_eq!(None, FiniteF32::try_new(f32::NEG_INFINITY));
        assert_eq!(0.5, f32::from(FiniteF32::from(0.5)));
    }

    #[test]
    #[should_panic(expected = "value should be finite")]
    fn test_finite_f32_rejects_nan() {
        let _ = FiniteF32::from(f32::NAN);
    }

    #[test]
    fn test_finite_f32_compares_with_f32() {
        let val = FiniteF32::from(0.5);

        assert_eq!(val, 0.5);
        assert_eq!(0.5, val);
        assert!(val > 0.25);
        assert!(0.75 > val);
        assert!(val < 0.75);
    }

    #[test]
    fn test_finite_f32_arithmetic() {
        let a = FiniteF32::from(3.0);
        let b = FiniteF32::from(2.0);

        // between two of them the guarantee holds
        assert_eq!(FiniteF32::from(5.0), a + b);
        assert_eq!(FiniteF32::from(1.0), a - b);
        assert_eq!(FiniteF32::from(6.0), a * b);
        assert_eq!(FiniteF32::from(1.5), a / b);

        // with a plain f32 it does not, so the result is a plain f32
        assert_eq!(4.0_f32, a + 1.0);
        assert_eq!(4.0_f32, 1.0 + a);
        assert_eq!(-3.0_f32, (-a).val());

        // ... which means these cannot panic
        assert!((f32::MAX + FiniteF32::from(f32::MAX)).is_infinite());
        assert!((FiniteF32::from(0.0) / 0.0).is_nan());
    }

    #[test]
    fn test_non_neg_f32_construction() {
        assert_eq!(Some(NonNegF32::from(0.0)), NonNegF32::try_new(0.0));
        assert_eq!(None, NonNegF32::try_new(-0.5));
        assert_eq!(None, NonNegF32::try_new(f32::NAN));
        assert_eq!(0.5, f32::from(NonNegF32::from(0.5)));
        assert_eq!(FiniteF32::from(0.5), FiniteF32::from(NonNegF32::from(0.5)));
    }

    #[test]
    #[should_panic(expected = "value should be finite and not negative")]
    fn test_non_neg_f32_rejects_negative() {
        let _ = NonNegF32::from(-0.5);
    }

    #[test]
    fn test_non_neg_f32_arithmetic() {
        let a = NonNegF32::from(3.0);
        let b = NonNegF32::from(2.0);

        // operations that stay in the range keep the type
        assert_eq!(NonNegF32::from(5.0), a + b);
        assert_eq!(NonNegF32::from(6.0), a * b);
        assert_eq!(NonNegF32::from(1.5), a / b);

        // ... the others fall back to the wider type
        assert_eq!(FiniteF32::from(-1.0), b - a);
        assert_eq!(FiniteF32::from(-3.0), -a);

        // ... and a plain f32 drops the guarantee entirely
        assert_eq!(1.0_f32, a - 2.0);
        assert_eq!(2.0_f32, 5.0 - a);
    }

    #[test]
    #[should_panic(expected = "value should be finite")]
    fn test_finite_f32_arithmetic_overflow_panics() {
        let max = FiniteF32::from(f32::MAX);
        let _ = max + max;
    }
}

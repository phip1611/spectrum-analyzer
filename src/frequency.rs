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
//! Module for the struct [`OrderableF32`] and the two
//! convenient type definitions [`Frequency`] and [`FrequencyValue`].

use core::cmp::Ordering;
use core::fmt::{Display, Formatter, Result};
use core::ops::{Add, Div, Mul, Sub};

/// A frequency in Hertz. A convenient wrapper type around `f32`.
pub type Frequency = OrderableF32;
/// The value of a [`Frequency`] in a frequency spectrum. Also called the
/// magnitude.
pub type FrequencyValue = OrderableF32;

/// Wrapper around [`f32`] that guarantees a valid number, hence, the number is
/// neither `NaN` or `infinite`. This makes the number orderable and sortable.
#[derive(Debug, Copy, Clone, Default)]
pub struct OrderableF32(f32);

impl OrderableF32 {
    #[inline]
    pub const fn val(&self) -> f32 {
        self.0
    }
}

impl From<f32> for OrderableF32 {
    #[inline]
    fn from(val: f32) -> Self {
        debug_assert!(!val.is_nan(), "NaN-values are not supported!");
        debug_assert!(!val.is_infinite(), "Infinite-values are not supported!");
        Self(val)
    }
}

impl Display for OrderableF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Ord for OrderableF32 {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for OrderableF32 {}

impl PartialEq for OrderableF32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal)
    }
}

impl PartialOrd for OrderableF32 {
    #[allow(clippy::float_cmp)]
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // self.cmp(other).is_eq()
        Some(if self.val() < other.val() {
            Ordering::Less
        } else if self.val() == other.val() {
            Ordering::Equal
        } else {
            Ordering::Greater
        })
    }
}

impl Add for OrderableF32 {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self::Output {
        (self.val() + other.val()).into()
    }
}

impl Sub for OrderableF32 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self::Output {
        (self.val() - other.val()).into()
    }
}

impl Mul for OrderableF32 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self::Output {
        (self.val() * other.val()).into()
    }
}

impl Div for OrderableF32 {
    type Output = Self;

    #[inline]
    fn div(self, other: Self) -> Self::Output {
        let quotient = self.val() / other.val();
        debug_assert!(!quotient.is_nan(), "NaN is not allowed");
        debug_assert!(!quotient.is_infinite(), "INFINITY is not allowed");
        quotient.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderablef32() {
        let f1: OrderableF32 = (2.0_f32).into();
        let f2: OrderableF32 = (-7.0_f32).into();

        let f3 = f1 + f2;
        let f4 = f1 - f2;

        assert_eq!(-5.0, f3.val(), "add must work");
        assert_eq!(9.0, f4.val(), "add must work");
        assert!(f2 < f1, "Compare must work");
        assert!(f1 > f2, "Compare must work");
        #[allow(clippy::eq_op)]
        {
            assert_eq!(f1, f1, "Equal must work");
        }
    }
}

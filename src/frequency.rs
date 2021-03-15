//! Module for the struct [`OrderableF32`] and the two
//! convenient type definitions [`Frequency`] and [`FrequencyValue`].

use core::fmt::{Display, Formatter, Result};
use core::cmp::Ordering;
use core::ops::{Add, Sub, Mul, Div};

/// A frequency. A convenient wrapper type around `f32`.
pub type Frequency = OrderableF32;
/// The value of a frequency in a frequency spectrum. Convenient wrapper around `f32`.
/// Not necessarily the magnitude of the complex numbers because scaling/normalization
/// functions could have been applied.
pub type FrequencyValue = OrderableF32;

/// Small convenient wrapper around `f32`.
/// Mainly required to make `f32` operable in a sorted tree map.
/// You should only use the type aliases `Frequency` and `FrequencyValue`.
#[derive(Debug, Copy, Clone)]
pub struct OrderableF32(f32);

impl OrderableF32 {

    #[inline(always)]
    pub fn val(&self) -> f32 {
        self.0
    }
}

impl From<f32> for OrderableF32 {

    #[inline(always)]
    fn from(val: f32) -> Self {
        Self(val)
    }
}

impl Display for OrderableF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Ord for OrderableF32 {

    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for OrderableF32 {}

impl PartialEq for OrderableF32 {

    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        // self.cmp(other).is_eq()
        match self.cmp(other) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}

impl PartialOrd for OrderableF32 {

    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // self.cmp(other).is_eq()
        Some(
            if self.val() < other.val() {
                Ordering::Less
            } else if self.val() == other.val() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        )
    }
}

impl Add for OrderableF32 {
    type Output = OrderableF32;

    #[inline(always)]
    fn add(self, other: Self) -> Self::Output {
        (self.val() + other.val()).into()
    }
}

impl Sub for OrderableF32 {
    type Output = OrderableF32;

    #[inline(always)]
    fn sub(self, other: Self) -> Self::Output {
        (self.val() - other.val()).into()
    }
}

impl Mul for OrderableF32 {
    type Output = OrderableF32;

    #[inline(always)]
    fn mul(self, other: Self) -> Self::Output {
        (self.val() * other.val()).into()
    }
}

impl Div for OrderableF32 {
    type Output = OrderableF32;

    #[inline(always)]
    fn div(self, other: Self) -> Self::Output {
        (self.val() / other.val()).into()
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
        assert_eq!(f1, f1, "Equal must work");
    }
}

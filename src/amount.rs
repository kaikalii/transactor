//! A type for quantifying balances
//!
//! See [`Amount`] for more information

use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

const DECIMAL_POINT_MUL: f64 = 10_000.0;

/// A fixed-point number for use in representing amounts of money
///
/// This type abstracts an integer as a fixed-point number to avoid floating-point errors,
/// which are not acceptable when dealing with money.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Amount(i64);

impl Amount {
    /// Attempt to create an amount from an `f64`
    pub fn from_f64(amount: f64) -> Option<Self> {
        let amount_multiplied = (amount * DECIMAL_POINT_MUL).round();
        if amount_multiplied > i64::MAX as f64
            || amount_multiplied < i64::MIN as f64
            || amount_multiplied.is_nan()
        {
            None
        } else {
            Some(Amount(amount_multiplied as i64))
        }
    }
    /// Get the amount as an `f64`
    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / DECIMAL_POINT_MUL
    }
}

impl fmt::Debug for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_f64().fmt(f)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_f64().fmt(f)
    }
}

impl Add for Amount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Amount(self.0 + rhs.0)
    }
}

impl Sub for Amount {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Amount(self.0 - rhs.0)
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl PartialEq<f64> for Amount {
    fn eq(&self, other: &f64) -> bool {
        &self.as_f64() == other
    }
}

impl PartialOrd<f64> for Amount {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.as_f64().partial_cmp(other)
    }
}

impl Neg for Amount {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Amount(-self.0)
    }
}

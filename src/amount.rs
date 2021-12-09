use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

const AMOUNT_DECIMAL_PLACES: u8 = 4;

/// A fixed-point amount of money
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Amount(i64);

impl Amount {
    /// Attempt to create an amount from an `f64`
    pub fn from_f64(amount: f64) -> Option<Self> {
        let amount_multiplied = (amount * 10f64.powf(AMOUNT_DECIMAL_PLACES as f64)).round();
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
        self.0 as f64 / 10f64.powf(AMOUNT_DECIMAL_PLACES as f64)
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

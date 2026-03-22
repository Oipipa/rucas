use std::fmt;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Number {
    Integer(BigInt),
    Rational(BigRational),
}

impl Number {
    pub fn integer(value: impl Into<BigInt>) -> Self {
        Self::Integer(value.into())
    }

    pub fn rational(numer: impl Into<BigInt>, denom: impl Into<BigInt>) -> Self {
        let denom = denom.into();
        assert!(
            !denom.is_zero(),
            "rational numbers require a non-zero denominator"
        );

        Self::from_big_rational(BigRational::new(numer.into(), denom))
    }

    pub fn from_big_rational(rational: BigRational) -> Self {
        if rational.denom().is_one() {
            Self::Integer(rational.numer().clone())
        } else {
            Self::Rational(rational)
        }
    }

    pub fn zero() -> Self {
        Self::integer(0)
    }

    pub fn one() -> Self {
        Self::integer(1)
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Integer(value) => value.is_zero(),
            Self::Rational(value) => value.is_zero(),
        }
    }

    pub fn is_one(&self) -> bool {
        match self {
            Self::Integer(value) => value.is_one(),
            Self::Rational(value) => value.is_one(),
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    pub fn is_negative(&self) -> bool {
        match self {
            Self::Integer(value) => value.is_negative(),
            Self::Rational(value) => value.is_negative(),
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self::from_big_rational(self.to_big_rational() + other.to_big_rational())
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self::from_big_rational(self.to_big_rational() - other.to_big_rational())
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self::from_big_rational(self.to_big_rational() * other.to_big_rational())
    }

    pub fn div(&self, other: &Self) -> Self {
        assert!(!other.is_zero(), "division by zero is undefined");
        Self::from_big_rational(self.to_big_rational() / other.to_big_rational())
    }

    pub fn neg(&self) -> Self {
        Self::from_big_rational(-self.to_big_rational())
    }

    pub fn powi(&self, exp: i64) -> Option<Self> {
        if exp == 0 {
            return Some(Self::one());
        }

        if exp < 0 && self.is_zero() {
            return None;
        }

        let magnitude = exp.unsigned_abs();
        if magnitude > u32::MAX as u64 {
            return None;
        }

        let magnitude = magnitude as u32;

        match self {
            Self::Integer(value) => {
                let powered = value.clone().pow(magnitude);
                if exp < 0 {
                    Some(Self::rational(BigInt::one(), powered))
                } else {
                    Some(Self::Integer(powered))
                }
            }
            Self::Rational(value) => {
                let numer = value.numer().clone().pow(magnitude);
                let denom = value.denom().clone().pow(magnitude);
                if exp < 0 {
                    Some(Self::rational(denom, numer))
                } else {
                    Some(Self::rational(numer, denom))
                }
            }
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => value.to_i64(),
            Self::Rational(_) => None,
        }
    }

    pub(crate) fn sqrt_exact(&self) -> Option<Self> {
        match self {
            Self::Integer(value) => {
                if value.is_negative() {
                    return None;
                }

                let sqrt = value.sqrt();
                (&sqrt * &sqrt == *value).then_some(Self::Integer(sqrt))
            }
            Self::Rational(value) => {
                if value.is_negative() {
                    return None;
                }

                let numer_sqrt = value.numer().sqrt();
                let denom_sqrt = value.denom().sqrt();

                if &numer_sqrt * &numer_sqrt == *value.numer()
                    && &denom_sqrt * &denom_sqrt == *value.denom()
                {
                    Some(Self::rational(numer_sqrt, denom_sqrt))
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn to_big_rational(&self) -> BigRational {
        match self {
            Self::Integer(value) => BigRational::from_integer(value.clone()),
            Self::Rational(value) => value.clone(),
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Rational(value) => write!(f, "{}/{}", value.numer(), value.denom()),
        }
    }
}

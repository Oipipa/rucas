use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

use crate::{
    Expr, Number,
    core::multiplicative::{ExactProduct, common_product_factor},
};

use super::{
    CollectedPolynomial, PolynomialAnalysisLimits, PolynomialCoefficient,
    SparseUnivariatePolynomial, UnivariatePolynomial, accumulate_term,
};

impl<C: PolynomialCoefficient> SparseUnivariatePolynomial<C> {
    pub(super) fn add(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        let mut coefficients = self.coefficients.clone();
        for (degree, coefficient) in &other.coefficients {
            accumulate_term(&mut coefficients, *degree, coefficient.clone());
        }

        within_term_limit(&coefficients, limits).then(|| Self {
            variable: self.variable.clone(),
            coefficients,
        })
    }

    pub(super) fn mul(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        let mut coefficients = BTreeMap::new();
        for (left_degree, left_coeff) in &self.coefficients {
            for (right_degree, right_coeff) in &other.coefficients {
                accumulate_term(
                    &mut coefficients,
                    left_degree + right_degree,
                    left_coeff.mul(right_coeff),
                );
            }
        }

        within_term_limit(&coefficients, limits).then(|| Self {
            variable: self.variable.clone(),
            coefficients,
        })
    }

    pub(super) fn pow(&self, exponent: usize, limits: PolynomialAnalysisLimits) -> Option<Self> {
        let mut power = exponent;
        let mut base = self.clone();
        let mut result = Self::constant(&self.variable, C::one());

        while power > 0 {
            if power % 2 == 1 {
                result = result.mul(&base, limits)?;
            }

            power /= 2;
            if power > 0 {
                base = base.mul(&base, limits)?;
            }
        }

        Some(result)
    }

    pub(crate) fn exponent_gcd(&self) -> Option<usize> {
        let mut gcd = Option::<usize>::None;

        for degree in self.coefficients.keys().copied().filter(|degree| *degree > 0) {
            gcd = Some(match gcd {
                Some(current) => current.gcd(&degree),
                None => degree,
            });
        }

        gcd.filter(|gcd| *gcd > 1)
    }

    pub(crate) fn divide_exponents(&self, divisor: usize) -> Option<Self> {
        if divisor == 0 {
            return None;
        }

        let coefficients = self
            .coefficients
            .iter()
            .map(|(&degree, coefficient)| {
                if degree % divisor != 0 {
                    return None;
                }

                Some((degree / divisor, coefficient.clone()))
            })
            .collect::<Option<Vec<_>>>()?;

        Some(Self::from_coefficients(self.variable.clone(), coefficients))
    }
}

impl SparseUnivariatePolynomial<Number> {
    pub(crate) fn content(&self) -> Option<Number> {
        if self.is_zero() {
            return None;
        }

        let mut denominator_lcm = BigInt::one();
        let mut scaled_gcd = BigInt::zero();

        for coefficient in self.coefficients.values() {
            let rational = coefficient.to_big_rational();
            denominator_lcm = denominator_lcm.lcm(rational.denom());
        }

        for coefficient in self.coefficients.values() {
            let rational = coefficient.to_big_rational();
            let scaled = (rational.numer() * (&denominator_lcm / rational.denom())).abs();

            scaled_gcd = if scaled_gcd.is_zero() {
                scaled
            } else {
                scaled_gcd.gcd(&scaled)
            };
        }

        Some(Number::rational(scaled_gcd, denominator_lcm))
    }

    pub(crate) fn primitive_part(&self) -> Option<Self> {
        let content = self.content()?;
        self.div_scalar(&content)
    }

    pub(crate) fn monic(&self) -> Option<Self> {
        let leading = self.leading_coefficient()?.clone();
        self.div_scalar(&leading)
    }

    pub(crate) fn integer_coefficients(&self) -> Option<BTreeMap<usize, BigInt>> {
        let mut coefficients = BTreeMap::new();

        for (&degree, coefficient) in &self.coefficients {
            let Number::Integer(value) = coefficient else {
                return None;
            };
            coefficients.insert(degree, value.clone());
        }

        Some(coefficients)
    }

    pub(crate) fn derivative(&self) -> Self {
        let mut coefficients = BTreeMap::new();

        for (&degree, coefficient) in &self.coefficients {
            if degree == 0 {
                continue;
            }

            accumulate_term(
                &mut coefficients,
                degree - 1,
                coefficient.mul(&Number::integer(degree)),
            );
        }

        Self {
            variable: self.variable.clone(),
            coefficients,
        }
    }

    pub(crate) fn evaluate(&self, point: &Number) -> Number {
        let Some(degree) = self.degree() else {
            return Number::zero();
        };

        let mut value = Number::zero();
        for current_degree in (0..=degree).rev() {
            value = value.mul(point).add(&self.coefficient(current_degree));
        }

        value
    }

    pub(crate) fn div_rem(&self, divisor: &Self) -> Option<(Self, Self)> {
        if self.variable != divisor.variable || divisor.is_zero() {
            return None;
        }

        if self.is_zero() {
            let zero = Self::constant(&self.variable, Number::zero());
            return Some((zero.clone(), zero));
        }

        let divisor_degree = divisor.degree()?;
        let divisor_leading = divisor.leading_coefficient()?.clone();
        let mut quotient = BTreeMap::new();
        let mut remainder = self.coefficients.clone();

        while let Some((remainder_degree, remainder_leading)) = remainder
            .iter()
            .next_back()
            .map(|(degree, coefficient)| (*degree, coefficient.clone()))
        {
            if remainder_degree < divisor_degree {
                break;
            }

            let factor_degree = remainder_degree - divisor_degree;
            let factor_coefficient = remainder_leading.div(&divisor_leading);

            accumulate_term(&mut quotient, factor_degree, factor_coefficient.clone());

            for (&degree, coefficient) in &divisor.coefficients {
                accumulate_term(
                    &mut remainder,
                    degree + factor_degree,
                    coefficient.mul(&factor_coefficient).neg(),
                );
            }
        }

        Some((
            Self {
                variable: self.variable.clone(),
                coefficients: quotient,
            },
            Self {
                variable: self.variable.clone(),
                coefficients: remainder,
            },
        ))
    }

    pub(crate) fn exact_div(&self, divisor: &Self) -> Option<Self> {
        let (quotient, remainder) = self.div_rem(divisor)?;
        remainder.is_zero().then_some(quotient)
    }

    pub(crate) fn gcd(&self, other: &Self) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        if self.is_zero() {
            return if other.is_zero() {
                Some(other.clone())
            } else {
                other.monic()
            };
        }

        if other.is_zero() {
            return self.monic();
        }

        let mut left = self.clone();
        let mut right = other.clone();

        while !right.is_zero() {
            let (_, remainder) = left.div_rem(&right)?;
            left = right;
            right = remainder;
        }

        left.monic()
    }

    pub(crate) fn divide_by_linear_factor(&self, root: &Number) -> Option<Self> {
        let linear_factor =
            Self::from_coefficients(self.variable.clone(), [(1, Number::one()), (0, root.neg())]);

        self.exact_div(&linear_factor)
    }

    pub(crate) fn deflate_linear_root(&self, root: &Number) -> Option<(usize, Self)> {
        let mut multiplicity = 0usize;
        let mut quotient = self.clone();

        while let Some(next) = quotient.divide_by_linear_factor(root) {
            quotient = next;
            multiplicity += 1;
        }

        (multiplicity > 0).then_some((multiplicity, quotient))
    }

    pub(crate) fn square_free_decomposition(&self) -> Option<Vec<(Self, usize)>> {
        if self.is_zero() {
            return None;
        }

        let normalized = self.monic()?;
        if normalized.degree().is_none_or(|degree| degree == 0) {
            return Some(Vec::new());
        }

        let derivative = normalized.derivative();
        let mut shared = normalized.gcd(&derivative)?;
        let mut remaining = normalized.exact_div(&shared)?;
        let mut factors = Vec::new();
        let mut multiplicity = 1usize;

        while !remaining.is_one() {
            let overlap = remaining.gcd(&shared)?;
            let square_free = remaining.exact_div(&overlap)?;

            if !square_free.is_one() {
                factors.push((square_free, multiplicity));
            }

            remaining = overlap;
            shared = shared.exact_div(&remaining)?;
            multiplicity += 1;
        }

        Some(factors)
    }

    pub(crate) fn div_scalar(&self, scalar: &Number) -> Option<Self> {
        if scalar.is_zero() {
            return None;
        }

        Some(Self::from_coefficients(
            self.variable.clone(),
            self.coefficients
                .iter()
                .map(|(&degree, coefficient)| (degree, coefficient.div(scalar))),
        ))
    }
}

impl SparseUnivariatePolynomial<Expr> {
    pub(crate) fn content(&self) -> Option<Expr> {
        if self.is_zero() {
            return None;
        }

        Some(
            common_product_factor(
                self.coefficients
                    .values()
                    .map(ExactProduct::from_expr)
                    .collect::<Vec<_>>()
                    .iter(),
            )?
            .to_expr(),
        )
    }

    pub(crate) fn primitive_part(&self) -> Option<Self> {
        let content = self.content()?;
        self.div_expr_factor(&content)
    }

    pub(crate) fn div_expr_factor(&self, factor: &Expr) -> Option<Self> {
        if factor.is_zero() {
            return None;
        }

        let divisor = ExactProduct::from_expr(factor);
        let coefficients = self
            .coefficients
            .iter()
            .map(|(&degree, coefficient)| {
                Some((
                    degree,
                    ExactProduct::from_expr(coefficient)
                        .divide_by(&divisor)?
                        .to_expr(),
                ))
            })
            .collect::<Option<Vec<_>>>()?;

        Some(Self::from_coefficients(self.variable.clone(), coefficients))
    }

    pub(crate) fn mul_expr_factor(&self, factor: &Expr) -> Self {
        Self::from_coefficients(
            self.variable.clone(),
            self.coefficients.iter().map(|(&degree, coefficient)| {
                (degree, Expr::product([coefficient.clone(), factor.clone()]))
            }),
        )
    }

    pub(crate) fn numeric_coefficients(&self) -> Option<UnivariatePolynomial> {
        Some(UnivariatePolynomial::from_coefficients(
            self.variable.clone(),
            self.coefficients
                .iter()
                .map(|(&degree, coefficient)| Some((degree, coefficient.as_number()?.clone())))
                .collect::<Option<Vec<_>>>()?,
        ))
    }

    pub(crate) fn from_numeric_coefficients(polynomial: &UnivariatePolynomial) -> Self {
        CollectedPolynomial::from_coefficients(
            polynomial.variable().clone(),
            polynomial
                .coefficients()
                .iter()
                .map(|(&degree, coefficient)| (degree, Expr::number(coefficient.clone()))),
        )
    }
}

fn within_term_limit(
    coefficients: &BTreeMap<usize, impl PolynomialCoefficient>,
    limits: PolynomialAnalysisLimits,
) -> bool {
    coefficients.len() <= limits.max_terms
}

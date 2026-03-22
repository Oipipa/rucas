mod algebra;
mod analysis;
mod collected;
mod expr;
mod rational;
mod score;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::{Expr, Number, Symbol};

const DEFAULT_MAX_TERMS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PolynomialAnalysisLimits {
    pub(crate) max_terms: usize,
}

impl Default for PolynomialAnalysisLimits {
    fn default() -> Self {
        Self {
            max_terms: DEFAULT_MAX_TERMS,
        }
    }
}

pub(crate) trait PolynomialCoefficient: Clone + Eq {
    fn zero() -> Self;
    fn one() -> Self;
    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;
    fn add(&self, other: &Self) -> Self;
    fn mul(&self, other: &Self) -> Self;
}

impl PolynomialCoefficient for Number {
    fn zero() -> Self {
        Self::zero()
    }

    fn one() -> Self {
        Self::one()
    }

    fn is_zero(&self) -> bool {
        self.is_zero()
    }

    fn is_one(&self) -> bool {
        self.is_one()
    }

    fn add(&self, other: &Self) -> Self {
        self.add(other)
    }

    fn mul(&self, other: &Self) -> Self {
        self.mul(other)
    }
}

impl PolynomialCoefficient for Expr {
    fn zero() -> Self {
        Self::zero()
    }

    fn one() -> Self {
        Self::one()
    }

    fn is_zero(&self) -> bool {
        self.is_zero()
    }

    fn is_one(&self) -> bool {
        self.is_one()
    }

    fn add(&self, other: &Self) -> Self {
        Expr::sum([self.clone(), other.clone()])
    }

    fn mul(&self, other: &Self) -> Self {
        Expr::product([self.clone(), other.clone()])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SparseUnivariatePolynomial<C> {
    variable: Symbol,
    coefficients: BTreeMap<usize, C>,
}

pub(crate) type UnivariatePolynomial = SparseUnivariatePolynomial<Number>;
pub(crate) type CollectedPolynomial = SparseUnivariatePolynomial<Expr>;

pub(crate) use collected::collect_polynomial_sum;
pub(crate) use expr::monomial_expr;
pub(crate) use rational::collect_rational_expression;

impl<C: PolynomialCoefficient> SparseUnivariatePolynomial<C> {
    pub(crate) fn from_coefficients(
        variable: Symbol,
        coefficients: impl IntoIterator<Item = (usize, C)>,
    ) -> Self {
        let mut normalized = BTreeMap::new();

        for (degree, coefficient) in coefficients {
            accumulate_term(&mut normalized, degree, coefficient);
        }

        Self {
            variable,
            coefficients: normalized,
        }
    }

    pub(crate) fn variable(&self) -> &Symbol {
        &self.variable
    }

    pub(crate) fn degree(&self) -> Option<usize> {
        self.coefficients.keys().next_back().copied()
    }

    pub(crate) fn coefficient(&self, degree: usize) -> C {
        self.coefficients
            .get(&degree)
            .cloned()
            .unwrap_or_else(C::zero)
    }

    pub(crate) fn leading_coefficient(&self) -> Option<&C> {
        self.degree()
            .and_then(|degree| self.coefficients.get(&degree))
    }

    pub(crate) fn coefficients(&self) -> &BTreeMap<usize, C> {
        &self.coefficients
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }

    pub(crate) fn is_one(&self) -> bool {
        self.degree() == Some(0) && self.coefficient(0).is_one()
    }

    fn constant(variable: &Symbol, coefficient: C) -> Self {
        Self::from_coefficients(variable.clone(), [(0, coefficient)])
    }

    fn monomial(variable: &Symbol, degree: usize, coefficient: C) -> Self {
        Self::from_coefficients(variable.clone(), [(degree, coefficient)])
    }
}

fn accumulate_term<C: PolynomialCoefficient>(
    coefficients: &mut BTreeMap<usize, C>,
    degree: usize,
    coefficient: C,
) {
    if coefficient.is_zero() {
        return;
    }

    coefficients
        .entry(degree)
        .and_modify(|current| *current = current.add(&coefficient))
        .or_insert(coefficient);

    if coefficients
        .get(&degree)
        .is_some_and(PolynomialCoefficient::is_zero)
    {
        coefficients.remove(&degree);
    }
}

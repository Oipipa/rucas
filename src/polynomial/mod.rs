mod algebra;
mod analysis;
mod expr;
mod rational;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::{Number, Symbol};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnivariatePolynomial {
    variable: Symbol,
    coefficients: BTreeMap<usize, Number>,
}

pub(crate) use expr::monomial_expr;
pub(crate) use rational::UnivariateRationalFunction;

impl UnivariatePolynomial {
    pub(crate) fn from_coefficients(
        variable: Symbol,
        coefficients: impl IntoIterator<Item = (usize, Number)>,
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

    pub(crate) fn coefficient(&self, degree: usize) -> Number {
        self.coefficients
            .get(&degree)
            .cloned()
            .unwrap_or_else(Number::zero)
    }

    pub(crate) fn leading_coefficient(&self) -> Option<&Number> {
        self.degree()
            .and_then(|degree| self.coefficients.get(&degree))
    }

    pub(crate) fn coefficients(&self) -> &BTreeMap<usize, Number> {
        &self.coefficients
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }

    pub(crate) fn is_one(&self) -> bool {
        self.degree() == Some(0) && self.coefficient(0).is_one()
    }

    fn constant(variable: &Symbol, coefficient: Number) -> Self {
        Self::from_coefficients(variable.clone(), [(0, coefficient)])
    }

    fn monomial(variable: &Symbol, degree: usize, coefficient: Number) -> Self {
        Self::from_coefficients(variable.clone(), [(degree, coefficient)])
    }
}

fn accumulate_term(coefficients: &mut BTreeMap<usize, Number>, degree: usize, coefficient: Number) {
    if coefficient.is_zero() {
        return;
    }

    coefficients
        .entry(degree)
        .and_modify(|current| *current = current.add(&coefficient))
        .or_insert(coefficient);

    if coefficients.get(&degree).is_some_and(Number::is_zero) {
        coefficients.remove(&degree);
    }
}

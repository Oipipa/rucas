use crate::{Expr, ExprKind, Number, Symbol};

use super::{PolynomialAnalysisLimits, UnivariatePolynomial, analysis::infer_variable};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnivariateRationalFunction {
    variable: Symbol,
    numerator: UnivariatePolynomial,
    denominator: UnivariatePolynomial,
}

impl UnivariateRationalFunction {
    pub(crate) fn from_expr(expr: &Expr) -> Option<Self> {
        Self::from_expr_with_limits(expr, PolynomialAnalysisLimits::default())
    }

    pub(crate) fn from_expr_with_limits(
        expr: &Expr,
        limits: PolynomialAnalysisLimits,
    ) -> Option<Self> {
        let variable = infer_variable(expr)?;
        Self::from_expr_with_variable_and_limits(expr, &variable, limits)
    }

    pub(crate) fn from_expr_with_variable_and_limits(
        expr: &Expr,
        variable: &Symbol,
        limits: PolynomialAnalysisLimits,
    ) -> Option<Self> {
        analyze(expr, variable, limits)
    }

    #[cfg(test)]
    pub(crate) fn numerator(&self) -> &UnivariatePolynomial {
        &self.numerator
    }

    #[cfg(test)]
    pub(crate) fn denominator(&self) -> &UnivariatePolynomial {
        &self.denominator
    }

    pub(crate) fn to_expr(&self) -> Expr {
        if self.denominator.is_one() {
            self.numerator.to_expr()
        } else {
            Expr::product([
                self.numerator.to_expr(),
                Expr::pow(self.denominator.to_expr(), Expr::integer(-1)),
            ])
        }
    }

    fn from_parts(
        variable: Symbol,
        numerator: UnivariatePolynomial,
        denominator: UnivariatePolynomial,
    ) -> Option<Self> {
        Self {
            variable,
            numerator,
            denominator,
        }
        .normalize()
    }

    fn polynomial(variable: &Symbol, polynomial: UnivariatePolynomial) -> Option<Self> {
        Self::from_parts(
            variable.clone(),
            polynomial,
            constant_polynomial(variable, Number::one()),
        )
    }

    fn add(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        let left = self.numerator.mul(&other.denominator, limits)?;
        let right = other.numerator.mul(&self.denominator, limits)?;
        let numerator = left.add(&right, limits)?;
        let denominator = self.denominator.mul(&other.denominator, limits)?;

        Self::from_parts(self.variable.clone(), numerator, denominator)
    }

    fn mul(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        let numerator = self.numerator.mul(&other.numerator, limits)?;
        let denominator = self.denominator.mul(&other.denominator, limits)?;

        Self::from_parts(self.variable.clone(), numerator, denominator)
    }

    fn powi(&self, exponent: i64, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if exponent == 0 {
            return Self::from_parts(
                self.variable.clone(),
                constant_polynomial(&self.variable, Number::one()),
                constant_polynomial(&self.variable, Number::one()),
            );
        }

        let magnitude = exponent.unsigned_abs();
        if magnitude > usize::MAX as u64 {
            return None;
        }

        let magnitude = magnitude as usize;
        let numerator = self.numerator.pow(magnitude, limits)?;
        let denominator = self.denominator.pow(magnitude, limits)?;

        if exponent > 0 {
            Self::from_parts(self.variable.clone(), numerator, denominator)
        } else {
            Self::from_parts(self.variable.clone(), denominator, numerator)
        }
    }

    fn normalize(mut self) -> Option<Self> {
        if self.denominator.is_zero() {
            return None;
        }

        if self.numerator.is_zero() {
            self.denominator = constant_polynomial(&self.variable, Number::one());
            return Some(self);
        }

        let gcd = self.numerator.gcd(&self.denominator)?;
        self.numerator = self.numerator.exact_div(&gcd)?;
        self.denominator = self.denominator.exact_div(&gcd)?;

        let denominator_leading = self.denominator.leading_coefficient()?.clone();
        self.numerator = self.numerator.div_scalar(&denominator_leading)?;
        self.denominator = self.denominator.div_scalar(&denominator_leading)?;

        if self.numerator.is_zero() {
            self.denominator = constant_polynomial(&self.variable, Number::one());
        }

        Some(self)
    }
}

fn analyze(
    expr: &Expr,
    variable: &Symbol,
    limits: PolynomialAnalysisLimits,
) -> Option<UnivariateRationalFunction> {
    match expr.kind() {
        ExprKind::Add(terms) => {
            let mut rational = UnivariateRationalFunction::polynomial(
                variable,
                constant_polynomial(variable, Number::zero()),
            )?;

            for term in terms {
                rational = rational.add(&analyze(term, variable, limits)?, limits)?;
            }

            Some(rational)
        }
        ExprKind::Mul(factors) => {
            let mut rational = UnivariateRationalFunction::polynomial(
                variable,
                constant_polynomial(variable, Number::one()),
            )?;

            for factor in factors {
                rational = rational.mul(&analyze(factor, variable, limits)?, limits)?;
            }

            Some(rational)
        }
        ExprKind::Pow { base, exp } => {
            let exponent = exp.as_number()?.as_i64()?;
            analyze(base, variable, limits)?.powi(exponent, limits)
        }
        ExprKind::Number(_) | ExprKind::Symbol(_) => UnivariateRationalFunction::polynomial(
            variable,
            UnivariatePolynomial::from_expr_with_variable_and_limits(expr, variable, limits)?,
        ),
        ExprKind::Call { .. } | ExprKind::Derivative(_) | ExprKind::Integral(_) => None,
    }
}

fn constant_polynomial(variable: &Symbol, value: Number) -> UnivariatePolynomial {
    UnivariatePolynomial::from_coefficients(variable.clone(), [(0, value)])
}

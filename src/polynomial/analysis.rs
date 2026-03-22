use std::collections::BTreeSet;

use crate::{Expr, ExprKind, Number, Symbol};

use super::{PolynomialAnalysisLimits, SparseUnivariatePolynomial, UnivariatePolynomial};

impl SparseUnivariatePolynomial<Number> {
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
}

fn analyze(
    expr: &Expr,
    variable: &Symbol,
    limits: PolynomialAnalysisLimits,
) -> Option<UnivariatePolynomial> {
    match expr.kind() {
        ExprKind::Number(number) => Some(UnivariatePolynomial::constant(variable, number.clone())),
        ExprKind::Symbol(symbol) if symbol == variable => {
            Some(UnivariatePolynomial::monomial(variable, 1, Number::one()))
        }
        ExprKind::Symbol(_) => None,
        ExprKind::Add(terms) => {
            let mut polynomial = UnivariatePolynomial::constant(variable, Number::zero());

            for term in terms {
                polynomial = polynomial.add(&analyze(term, variable, limits)?, limits)?;
            }

            Some(polynomial)
        }
        ExprKind::Mul(factors) => {
            let mut polynomial = UnivariatePolynomial::constant(variable, Number::one());

            for factor in factors {
                polynomial = polynomial.mul(&analyze(factor, variable, limits)?, limits)?;
            }

            Some(polynomial)
        }
        ExprKind::Pow { base, exp } => {
            let exponent = exp.as_number()?.as_i64()?;
            if exponent < 0 {
                return None;
            }

            analyze(base, variable, limits)?.pow(exponent as usize, limits)
        }
        ExprKind::Call { .. } | ExprKind::Derivative(_) | ExprKind::Integral(_) => None,
    }
}

pub(super) fn infer_variable(expr: &Expr) -> Option<Symbol> {
    let symbols = symbols(expr);

    (symbols.len() == 1)
        .then(|| symbols.into_iter().next())
        .flatten()
}

pub(super) fn symbols(expr: &Expr) -> BTreeSet<Symbol> {
    let mut symbols = BTreeSet::new();
    collect_symbols(expr, &mut symbols);
    symbols
}

fn collect_symbols(expr: &Expr, symbols: &mut BTreeSet<Symbol>) {
    match expr.kind() {
        ExprKind::Number(_) => {}
        ExprKind::Symbol(symbol) => {
            symbols.insert(symbol.clone());
        }
        ExprKind::Add(terms) | ExprKind::Mul(terms) => {
            for term in terms {
                collect_symbols(term, symbols);
            }
        }
        ExprKind::Pow { base, exp } => {
            collect_symbols(base, symbols);
            collect_symbols(exp, symbols);
        }
        ExprKind::Call { args, .. } => {
            for arg in args {
                collect_symbols(arg, symbols);
            }
        }
        ExprKind::Derivative(derivative) => {
            collect_symbols(&derivative.expr, symbols);
            symbols.insert(derivative.variable.clone());
        }
        ExprKind::Integral(integral) => {
            collect_symbols(&integral.expr, symbols);
            symbols.insert(integral.variable.clone());
        }
    }
}

use crate::{Expr, ExprKind, Symbol, core::visit::any};

use super::{
    CollectedPolynomial, PolynomialAnalysisLimits, SparseUnivariatePolynomial, analysis::symbols,
    score::expression_score,
};

impl SparseUnivariatePolynomial<Expr> {
    pub(crate) fn from_expr_with_variable_and_limits(
        expr: &Expr,
        variable: &Symbol,
        limits: PolynomialAnalysisLimits,
    ) -> Option<Self> {
        analyze(expr, variable, limits)
    }
}

pub(crate) fn collect_polynomial_sum(expr: &Expr) -> Option<Expr> {
    let ExprKind::Add(_) = expr.kind() else {
        return None;
    };

    let mut best: Option<_> = None;

    for variable in symbols(expr) {
        let Some(candidate) = CollectedPolynomial::from_expr_with_variable_and_limits(
            expr,
            &variable,
            PolynomialAnalysisLimits::default(),
        )
        .map(|polynomial| polynomial.to_expr()) else {
            continue;
        };

        let candidate_score = expression_score(&candidate);
        match &best {
            Some((current_score, _)) if candidate_score >= *current_score => {}
            _ => best = Some((candidate_score, candidate)),
        }
    }

    best.and_then(|(_, candidate)| (candidate != *expr).then_some(candidate))
}

fn analyze(
    expr: &Expr,
    variable: &Symbol,
    limits: PolynomialAnalysisLimits,
) -> Option<CollectedPolynomial> {
    if !depends_on_variable(expr, variable) {
        return Some(CollectedPolynomial::constant(variable, expr.clone()));
    }

    match expr.kind() {
        ExprKind::Number(number) => Some(CollectedPolynomial::constant(
            variable,
            Expr::number(number.clone()),
        )),
        ExprKind::Symbol(symbol) if symbol == variable => {
            Some(CollectedPolynomial::monomial(variable, 1, Expr::one()))
        }
        ExprKind::Symbol(_) => Some(CollectedPolynomial::constant(variable, expr.clone())),
        ExprKind::Add(terms) => {
            let mut polynomial = CollectedPolynomial::constant(variable, Expr::zero());

            for term in terms {
                polynomial = polynomial.add(&analyze(term, variable, limits)?, limits)?;
            }

            Some(polynomial)
        }
        ExprKind::Mul(factors) => {
            let mut polynomial = CollectedPolynomial::constant(variable, Expr::one());

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

fn depends_on_variable(expr: &Expr, variable: &Symbol) -> bool {
    any(expr, &mut |candidate| match candidate.kind() {
        ExprKind::Symbol(symbol) => symbol == variable,
        _ => false,
    })
}

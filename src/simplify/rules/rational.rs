use crate::{
    Expr, ExprKind,
    context::EngineContext,
    polynomial::collect_rational_expression,
    rewrite::{RewriteEngine, RewriteRule},
};

pub(super) fn install(engine: &mut RewriteEngine) {
    engine.push_rule(CancelPolynomialRationalsRule);
}

struct CancelPolynomialRationalsRule;

impl RewriteRule for CancelPolynomialRationalsRule {
    fn name(&self) -> &'static str {
        "cancel-polynomial-rationals"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        if !has_explicit_denominator(expr) {
            return None;
        }

        collect_rational_expression(expr)
    }
}

fn has_explicit_denominator(expr: &Expr) -> bool {
    match expr.kind() {
        ExprKind::Pow { exp, .. } => exp
            .as_number()
            .and_then(crate::Number::as_i64)
            .is_some_and(|value| value < 0),
        ExprKind::Add(terms) | ExprKind::Mul(terms) => terms.iter().any(has_explicit_denominator),
        _ => false,
    }
}

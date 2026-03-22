use crate::{
    Expr, ExprKind,
    context::EngineContext,
    polynomial::UnivariatePolynomial,
    rewrite::{RewriteEngine, RewriteRule},
};

pub(super) fn install(engine: &mut RewriteEngine) {
    engine.push_rule(CollectPolynomialSumRule);
}

struct CollectPolynomialSumRule;

impl RewriteRule for CollectPolynomialSumRule {
    fn name(&self) -> &'static str {
        "collect-polynomial-sums"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        let ExprKind::Add(_) = expr.kind() else {
            return None;
        };

        let normalized = UnivariatePolynomial::from_expr(expr)?.to_expr();
        (normalized != *expr).then_some(normalized)
    }
}

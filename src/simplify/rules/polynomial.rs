use crate::{
    Expr, ExprKind,
    context::EngineContext,
    polynomial::collect_polynomial_sum,
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

        collect_polynomial_sum(expr)
    }
}

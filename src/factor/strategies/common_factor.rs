use crate::{
    Expr, ExprKind,
    context::EngineContext,
    core::multiplicative::{ExactProduct, common_product_factor},
    factor::{FactorizationResult, FactorizationStatus, FactorizationStrategy, Factorizer},
};

pub(super) fn install(factorizer: &mut Factorizer) {
    factorizer.push_strategy(CommonFactorStrategy);
}

struct CommonFactorStrategy;

impl FactorizationStrategy for CommonFactorStrategy {
    fn name(&self) -> &'static str {
        "common-factor"
    }

    fn try_factor(&self, expr: &Expr, _ctx: &EngineContext) -> Option<FactorizationResult> {
        let ExprKind::Add(terms) = expr.kind() else {
            return None;
        };

        if terms.len() < 2 {
            return None;
        }

        let decomposed_terms: Vec<_> = terms.iter().map(ExactProduct::from_expr).collect();
        let common_factor = common_product_factor(decomposed_terms.iter())?;

        if common_factor.to_expr().is_one() {
            return None;
        }

        let residual = Expr::sum(decomposed_terms.iter().map(|term| {
            term.divide_by(&common_factor)
                .expect("common factor must divide every decomposed term")
                .to_expr()
        }));
        let factored = Expr::product([common_factor.to_expr(), residual]);

        (factored != *expr).then(|| FactorizationResult {
            expr: factored,
            status: FactorizationStatus::Factored,
            steps: vec!["common factor extracted".to_string()],
        })
    }
}

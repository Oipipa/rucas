use std::collections::BTreeMap;

use crate::{
    Expr, ExprKind, Number,
    context::EngineContext,
    rewrite::{RewriteEngine, RewriteRule},
};

pub(super) fn install(engine: &mut RewriteEngine) {
    engine.push_rule(CombineIntegerLikeFactorsRule);
    engine.push_rule(CollectLikeTermsRule);
}

struct CombineIntegerLikeFactorsRule;

impl RewriteRule for CombineIntegerLikeFactorsRule {
    fn name(&self) -> &'static str {
        "combine-integer-like-factors"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        let ExprKind::Mul(factors) = expr.kind() else {
            return None;
        };

        let mut rebuilt = Vec::new();
        let mut residual = Vec::new();
        let mut exponents = BTreeMap::<Expr, Number>::new();

        for factor in factors {
            if factor.as_number().is_some() {
                rebuilt.push(factor.clone());
                continue;
            }

            match integer_factor_power(factor) {
                Some((base, exp)) => {
                    exponents
                        .entry(base)
                        .and_modify(|current| *current = current.add(&exp))
                        .or_insert(exp);
                }
                None => residual.push(factor.clone()),
            }
        }

        rebuilt.extend(exponents.into_iter().filter_map(|(base, exp)| {
            if exp.is_zero() {
                None
            } else if exp.is_one() {
                Some(base)
            } else {
                Some(Expr::pow(base, Expr::number(exp)))
            }
        }));
        rebuilt.extend(residual);

        let simplified = Expr::product(rebuilt);
        (simplified != *expr).then_some(simplified)
    }
}

struct CollectLikeTermsRule;

impl RewriteRule for CollectLikeTermsRule {
    fn name(&self) -> &'static str {
        "collect-like-terms"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        let ExprKind::Add(terms) = expr.kind() else {
            return None;
        };

        let mut rebuilt = Vec::new();
        let mut numeric = Option::<Number>::None;
        let mut coefficients = BTreeMap::<Expr, Number>::new();

        for term in terms {
            if let Some(number) = term.as_number() {
                numeric = Some(match numeric {
                    Some(current) => current.add(number),
                    None => number.clone(),
                });
                continue;
            }

            let (body, coeff) = term_coefficient(term);
            coefficients
                .entry(body)
                .and_modify(|current| *current = current.add(&coeff))
                .or_insert(coeff);
        }

        if let Some(number) = numeric.filter(|number| !number.is_zero()) {
            rebuilt.push(Expr::number(number));
        }

        rebuilt.extend(coefficients.into_iter().filter_map(|(body, coeff)| {
            if coeff.is_zero() {
                None
            } else if coeff.is_one() {
                Some(body)
            } else {
                Some(Expr::product([Expr::number(coeff), body]))
            }
        }));

        let simplified = Expr::sum(rebuilt);
        (simplified != *expr).then_some(simplified)
    }
}

fn integer_factor_power(factor: &Expr) -> Option<(Expr, Number)> {
    match factor.kind() {
        ExprKind::Pow { base, exp } => exp
            .as_number()
            .filter(|number| number.is_integer())
            .map(|number| (base.clone(), number.clone())),
        ExprKind::Number(_) => None,
        _ => Some((factor.clone(), Number::one())),
    }
}

fn term_coefficient(term: &Expr) -> (Expr, Number) {
    match term.kind() {
        ExprKind::Mul(factors) => match factors.first().and_then(Expr::as_number) {
            Some(number) => (
                Expr::product(factors.iter().skip(1).cloned()),
                number.clone(),
            ),
            None => (term.clone(), Number::one()),
        },
        _ => (term.clone(), Number::one()),
    }
}

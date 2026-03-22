use crate::{Expr, ExprKind};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ExpressionScore {
    explicit_denominators: usize,
    top_level_terms: usize,
    node_count: usize,
    expr: Expr,
}

pub(crate) fn expression_score(expr: &Expr) -> ExpressionScore {
    ExpressionScore {
        explicit_denominators: explicit_denominator_count(expr),
        top_level_terms: top_level_terms(expr),
        node_count: node_count(expr),
        expr: expr.clone(),
    }
}

fn explicit_denominator_count(expr: &Expr) -> usize {
    let child_count = match expr.kind() {
        ExprKind::Number(_) | ExprKind::Symbol(_) => 0,
        ExprKind::Add(terms) | ExprKind::Mul(terms) => {
            terms.iter().map(explicit_denominator_count).sum()
        }
        ExprKind::Pow { base, exp } => {
            explicit_denominator_here(exp)
                + explicit_denominator_count(base)
                + explicit_denominator_count(exp)
        }
        ExprKind::Call { args, .. } => args.iter().map(explicit_denominator_count).sum(),
        ExprKind::Derivative(derivative) => explicit_denominator_count(&derivative.expr),
        ExprKind::Integral(integral) => explicit_denominator_count(&integral.expr),
    };

    child_count
}

fn explicit_denominator_here(exp: &Expr) -> usize {
    exp.as_number()
        .and_then(crate::Number::as_i64)
        .is_some_and(|value| value < 0) as usize
}

fn top_level_terms(expr: &Expr) -> usize {
    match expr.kind() {
        ExprKind::Add(terms) => terms.len(),
        _ => 1,
    }
}

fn node_count(expr: &Expr) -> usize {
    let child_count = match expr.kind() {
        ExprKind::Number(_) | ExprKind::Symbol(_) => 0,
        ExprKind::Add(terms) | ExprKind::Mul(terms) => terms.iter().map(node_count).sum(),
        ExprKind::Pow { base, exp } => node_count(base) + node_count(exp),
        ExprKind::Call { args, .. } => args.iter().map(node_count).sum(),
        ExprKind::Derivative(derivative) => node_count(&derivative.expr),
        ExprKind::Integral(integral) => node_count(&integral.expr),
    };

    1 + child_count
}

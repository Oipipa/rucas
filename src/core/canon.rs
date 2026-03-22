use super::{Expr, ExprKind, Number};

pub fn sum(terms: impl IntoIterator<Item = Expr>) -> Expr {
    let mut pending: Vec<Expr> = terms.into_iter().collect();
    let mut flattened = Vec::new();
    let mut numeric = Option::<Number>::None;

    while let Some(term) = pending.pop() {
        match term.kind() {
            ExprKind::Add(children) => pending.extend(children.iter().cloned()),
            ExprKind::Number(number) => {
                numeric = Some(match numeric {
                    Some(current) => current.add(number),
                    None => number.clone(),
                });
            }
            _ => flattened.push(term),
        }
    }

    flattened.retain(|term| !term.is_zero());
    flattened.sort();

    if let Some(number) = numeric.filter(|number| !number.is_zero()) {
        flattened.insert(0, Expr::number(number));
    }

    match flattened.len() {
        0 => Expr::zero(),
        1 => flattened
            .pop()
            .expect("exactly one term after normalization"),
        _ => Expr::raw(ExprKind::Add(flattened)),
    }
}

pub fn product(factors: impl IntoIterator<Item = Expr>) -> Expr {
    let mut pending: Vec<Expr> = factors.into_iter().collect();
    let mut flattened = Vec::new();
    let mut numeric = Option::<Number>::None;

    while let Some(factor) = pending.pop() {
        match factor.kind() {
            ExprKind::Mul(children) => pending.extend(children.iter().cloned()),
            ExprKind::Number(number) => {
                numeric = Some(match numeric {
                    Some(current) => current.mul(number),
                    None => number.clone(),
                });
            }
            _ => flattened.push(factor),
        }
    }

    if numeric.as_ref().is_some_and(Number::is_zero) {
        return Expr::zero();
    }

    flattened.retain(|factor| !factor.is_one());
    flattened.sort();

    if let Some(number) = numeric.filter(|number| !number.is_one()) {
        flattened.insert(0, Expr::number(number));
    }

    match flattened.len() {
        0 => Expr::one(),
        1 => flattened
            .pop()
            .expect("exactly one factor after normalization"),
        _ => Expr::raw(ExprKind::Mul(flattened)),
    }
}

pub fn pow(base: Expr, exp: Expr) -> Expr {
    if exp.is_zero() {
        return Expr::one();
    }

    if exp.is_one() {
        return base;
    }

    if base.is_zero()
        && exp
            .as_number()
            .and_then(Number::as_i64)
            .is_some_and(|value| value > 0)
    {
        return Expr::zero();
    }

    if base.is_one() {
        return Expr::one();
    }

    if let (Some(number), Some(exp_value)) =
        (base.as_number(), exp.as_number().and_then(Number::as_i64))
        && let Some(powered) = number.powi(exp_value)
    {
        return Expr::number(powered);
    }

    if let ExprKind::Pow {
        base: inner_base,
        exp: inner_exp,
    } = base.kind()
        && exp
            .as_number()
            .and_then(Number::as_i64)
            .is_some_and(|value| value > 0)
    {
        return pow(inner_base.clone(), Expr::product([inner_exp.clone(), exp]));
    }

    Expr::raw(ExprKind::Pow { base, exp })
}

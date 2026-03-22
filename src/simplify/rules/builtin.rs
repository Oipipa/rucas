use crate::{
    BuiltinFunction, Expr, ExprKind, Function,
    context::EngineContext,
    rewrite::{RewriteEngine, RewriteRule},
};

pub(super) fn install(engine: &mut RewriteEngine) {
    engine.push_rule(BuiltinParityRule);
    engine.push_rule(ExponentialPowerRule);
    engine.push_rule(ExponentialProductRule);
    engine.push_rule(BuiltinIdentitiesRule);
}

struct BuiltinParityRule;

impl RewriteRule for BuiltinParityRule {
    fn name(&self) -> &'static str {
        "builtin-parity"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        let ExprKind::Call { function, args } = expr.kind() else {
            return None;
        };

        let [arg] = args.as_slice() else {
            return None;
        };

        let positive_arg = strip_leading_negative(arg)?;

        match function {
            Function::Builtin(BuiltinFunction::Sin) => Some(Expr::product([
                Expr::integer(-1),
                builtin_unary(BuiltinFunction::Sin, positive_arg),
            ])),
            Function::Builtin(BuiltinFunction::Cos) => {
                Some(builtin_unary(BuiltinFunction::Cos, positive_arg))
            }
            _ => None,
        }
    }
}

struct ExponentialPowerRule;

impl RewriteRule for ExponentialPowerRule {
    fn name(&self) -> &'static str {
        "exp-power"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        let ExprKind::Pow { base, exp } = expr.kind() else {
            return None;
        };

        let ExprKind::Call {
            function: Function::Builtin(BuiltinFunction::Exp),
            args,
        } = base.kind()
        else {
            return None;
        };

        let [arg] = args.as_slice() else {
            return None;
        };

        let number = exp.as_number().filter(|number| number.is_integer())?;

        Some(builtin_unary(
            BuiltinFunction::Exp,
            Expr::product([Expr::number(number.clone()), arg.clone()]),
        ))
    }
}

struct ExponentialProductRule;

impl RewriteRule for ExponentialProductRule {
    fn name(&self) -> &'static str {
        "exp-product"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        let ExprKind::Mul(factors) = expr.kind() else {
            return None;
        };

        let mut exp_args = Vec::new();
        let mut residual = Vec::new();

        for factor in factors {
            match factor.kind() {
                ExprKind::Call {
                    function: Function::Builtin(BuiltinFunction::Exp),
                    args,
                } if args.len() == 1 => exp_args.push(args[0].clone()),
                _ => residual.push(factor.clone()),
            }
        }

        if exp_args.len() < 2 {
            return None;
        }

        residual.push(builtin_unary(BuiltinFunction::Exp, Expr::sum(exp_args)));
        Some(Expr::product(residual))
    }
}

struct BuiltinIdentitiesRule;

impl RewriteRule for BuiltinIdentitiesRule {
    fn name(&self) -> &'static str {
        "builtin-identities"
    }

    fn apply(&self, expr: &Expr, _ctx: &EngineContext) -> Option<Expr> {
        let ExprKind::Call { function, args } = expr.kind() else {
            return None;
        };

        match (function, args.as_slice()) {
            (Function::Builtin(BuiltinFunction::Sin), [arg]) if arg.is_zero() => Some(Expr::zero()),
            (Function::Builtin(BuiltinFunction::Cos), [arg]) if arg.is_zero() => Some(Expr::one()),
            (Function::Builtin(BuiltinFunction::Exp), [arg]) if arg.is_zero() => Some(Expr::one()),
            (Function::Builtin(BuiltinFunction::Log), [arg]) if arg.is_one() => Some(Expr::zero()),
            _ => None,
        }
    }
}

fn builtin_unary(function: BuiltinFunction, arg: Expr) -> Expr {
    Expr::call(Function::Builtin(function), [arg])
}

fn strip_leading_negative(expr: &Expr) -> Option<Expr> {
    let ExprKind::Mul(factors) = expr.kind() else {
        return None;
    };

    let number = factors.first()?.as_number()?;
    if number != &crate::Number::integer(-1) {
        return None;
    }

    Some(Expr::product(factors.iter().skip(1).cloned()))
}

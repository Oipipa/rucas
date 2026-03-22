use crate::{
    BuiltinFunction, Expr, ExprKind, Function, Symbol, context::EngineContext, core::visit::any,
    simplify::Simplifier,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Differentiator;

impl Differentiator {
    pub fn new() -> Self {
        Self
    }

    pub fn differentiate(&self, expr: &Expr, variable: &Symbol, ctx: &EngineContext) -> Expr {
        let derived = self.differentiate_inner(expr, variable);

        if ctx.auto_simplify {
            Simplifier::default().simplify(derived, ctx)
        } else {
            derived
        }
    }

    fn differentiate_inner(&self, expr: &Expr, variable: &Symbol) -> Expr {
        match expr.kind() {
            ExprKind::Number(_) => Expr::zero(),
            ExprKind::Symbol(symbol) => {
                if symbol == variable {
                    Expr::one()
                } else {
                    Expr::zero()
                }
            }
            ExprKind::Add(terms) => self.differentiate_sum(terms, variable),
            ExprKind::Mul(factors) => self.differentiate_product(factors, variable),
            ExprKind::Pow { base, exp } => self.differentiate_power(expr, base, exp, variable),
            ExprKind::Call { function, args } => {
                self.differentiate_call(expr, function, args, variable)
            }
            ExprKind::Derivative(_) | ExprKind::Integral(_) => self.unsupported(expr, variable),
        }
    }

    fn differentiate_sum(&self, terms: &[Expr], variable: &Symbol) -> Expr {
        Expr::sum(
            terms
                .iter()
                .map(|term| self.differentiate_inner(term, variable)),
        )
    }

    fn differentiate_product(&self, factors: &[Expr], variable: &Symbol) -> Expr {
        Expr::sum(factors.iter().enumerate().filter_map(|(index, _)| {
            let derived_factor = self.differentiate_inner(&factors[index], variable);
            if derived_factor.is_zero() {
                return None;
            }

            Some(Expr::product(factors.iter().enumerate().map(
                |(factor_index, factor)| {
                    if factor_index == index {
                        derived_factor.clone()
                    } else {
                        factor.clone()
                    }
                },
            )))
        }))
    }

    fn differentiate_power(&self, expr: &Expr, base: &Expr, exp: &Expr, variable: &Symbol) -> Expr {
        let base_depends_on_variable = self.depends_on_variable(base, variable);
        let exp_depends_on_variable = self.depends_on_variable(exp, variable);

        match (base_depends_on_variable, exp_depends_on_variable) {
            (false, false) => Expr::zero(),
            (true, false) => {
                let base_derivative = self.differentiate_inner(base, variable);
                self.power_rule(base, exp, base_derivative)
            }
            (false, true) => self.differentiate_constant_base_power(expr, base, exp, variable),
            (true, true) => self.differentiate_general_power(expr, base, exp, variable),
        }
    }

    fn differentiate_constant_base_power(
        &self,
        expr: &Expr,
        base: &Expr,
        exp: &Expr,
        variable: &Symbol,
    ) -> Expr {
        if base.is_zero() {
            return self.unsupported(expr, variable);
        }

        let exp_derivative = self.differentiate_inner(exp, variable);
        Expr::product([expr.clone(), exp_derivative, self.log_of(base.clone())])
    }

    fn differentiate_general_power(
        &self,
        expr: &Expr,
        base: &Expr,
        exp: &Expr,
        variable: &Symbol,
    ) -> Expr {
        if base.is_zero() {
            return self.unsupported(expr, variable);
        }

        let exp_derivative = self.differentiate_inner(exp, variable);
        let base_derivative = self.differentiate_inner(base, variable);

        // Use logarithmic differentiation for the fully general case:
        // d(f^g) = f^g * (g' * log(f) + g * f' / f).
        let logarithmic_factor = Expr::sum([
            Expr::product([exp_derivative, self.log_of(base.clone())]),
            Expr::product([exp.clone(), base_derivative, self.reciprocal(base.clone())]),
        ]);

        Expr::product([expr.clone(), logarithmic_factor])
    }

    fn differentiate_call(
        &self,
        expr: &Expr,
        function: &Function,
        args: &[Expr],
        variable: &Symbol,
    ) -> Expr {
        match (function, args) {
            (Function::Builtin(function), [arg]) => {
                self.differentiate_builtin_call(*function, arg, variable)
            }
            _ => self.unsupported(expr, variable),
        }
    }

    fn differentiate_builtin_call(
        &self,
        function: BuiltinFunction,
        arg: &Expr,
        variable: &Symbol,
    ) -> Expr {
        let arg_derivative = self.differentiate_inner(arg, variable);

        let outer_derivative = match function {
            BuiltinFunction::Sin => Self::builtin_unary(BuiltinFunction::Cos, arg.clone()),
            BuiltinFunction::Cos => Expr::product([
                Expr::integer(-1),
                Self::builtin_unary(BuiltinFunction::Sin, arg.clone()),
            ]),
            BuiltinFunction::Exp => Self::builtin_unary(BuiltinFunction::Exp, arg.clone()),
            BuiltinFunction::Log => Expr::pow(arg.clone(), Expr::integer(-1)),
        };

        Expr::product([outer_derivative, arg_derivative])
    }

    fn power_rule(&self, base: &Expr, exp: &Expr, base_derivative: Expr) -> Expr {
        Expr::product([
            exp.clone(),
            Expr::pow(base.clone(), Expr::sum([exp.clone(), Expr::integer(-1)])),
            base_derivative,
        ])
    }

    fn builtin_unary(function: BuiltinFunction, arg: Expr) -> Expr {
        Expr::call(Function::Builtin(function), [arg])
    }

    fn log_of(&self, expr: Expr) -> Expr {
        Self::builtin_unary(BuiltinFunction::Log, expr)
    }

    fn reciprocal(&self, expr: Expr) -> Expr {
        Expr::pow(expr, Expr::integer(-1))
    }

    fn depends_on_variable(&self, expr: &Expr, variable: &Symbol) -> bool {
        any(expr, &mut |candidate| match candidate.kind() {
            ExprKind::Symbol(symbol) => symbol == variable,
            _ => false,
        })
    }

    fn unsupported(&self, expr: &Expr, variable: &Symbol) -> Expr {
        Expr::derivative(expr.clone(), variable.clone())
    }
}

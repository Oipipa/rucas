use crate::{Expr, ExprKind, Symbol, context::EngineContext, simplify::Simplifier};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Differentiator {
    pub allow_unevaluated: bool,
}

impl Default for Differentiator {
    fn default() -> Self {
        Self {
            allow_unevaluated: true,
        }
    }
}

impl Differentiator {
    pub fn new() -> Self {
        Self::default()
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
            ExprKind::Add(terms) => Expr::sum(
                terms
                    .iter()
                    .map(|term| self.differentiate_inner(term, variable)),
            ),
            ExprKind::Mul(_)
            | ExprKind::Pow { .. }
            | ExprKind::Call { .. }
            | ExprKind::Derivative(_)
            | ExprKind::Integral(_) => {
                if self.allow_unevaluated {
                    Expr::derivative(expr.clone(), variable.clone())
                } else {
                    Expr::zero()
                }
            }
        }
    }
}

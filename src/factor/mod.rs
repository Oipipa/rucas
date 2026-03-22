use std::collections::BTreeSet;

mod strategies;

use crate::{Expr, ExprKind, context::EngineContext};

use self::strategies::install_default_strategies;

pub trait FactorizationStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn try_factor(&self, expr: &Expr, ctx: &EngineContext) -> Option<FactorizationResult>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactorizationStatus {
    Factored,
    Unchanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactorizationResult {
    pub expr: Expr,
    pub status: FactorizationStatus,
    pub steps: Vec<String>,
}

impl FactorizationResult {
    pub fn factored(expr: Expr, step: impl Into<String>) -> Self {
        Self {
            expr,
            status: FactorizationStatus::Factored,
            steps: vec![step.into()],
        }
    }

    pub fn unchanged(expr: Expr) -> Self {
        Self {
            expr,
            status: FactorizationStatus::Unchanged,
            steps: Vec::new(),
        }
    }
}

pub struct Factorizer {
    strategies: Vec<Box<dyn FactorizationStrategy>>,
}

impl Default for Factorizer {
    fn default() -> Self {
        let mut factorizer = Self::empty();
        install_default_strategies(&mut factorizer);
        factorizer
    }
}

impl Factorizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn with_strategy(mut self, strategy: impl FactorizationStrategy + 'static) -> Self {
        self.push_strategy(strategy);
        self
    }

    pub fn push_strategy(&mut self, strategy: impl FactorizationStrategy + 'static) {
        self.strategies.push(Box::new(strategy));
    }

    pub fn factor(&self, expr: &Expr, ctx: &EngineContext) -> FactorizationResult {
        let mut steps = Vec::new();
        let factored = self.factor_expr(expr, ctx, &mut steps);

        if factored == *expr {
            return FactorizationResult::unchanged(factored);
        }

        FactorizationResult {
            expr: factored,
            status: FactorizationStatus::Factored,
            steps,
        }
    }

    fn factor_expr(&self, expr: &Expr, ctx: &EngineContext, steps: &mut Vec<String>) -> Expr {
        let rebuilt = self.factor_children(expr, ctx, steps);
        self.factor_node_to_fixpoint(rebuilt, ctx, steps)
    }

    fn factor_children(&self, expr: &Expr, ctx: &EngineContext, steps: &mut Vec<String>) -> Expr {
        match expr.kind() {
            ExprKind::Number(_) | ExprKind::Symbol(_) => expr.clone(),
            ExprKind::Add(terms) => {
                Expr::sum(terms.iter().map(|term| self.factor_expr(term, ctx, steps)))
            }
            ExprKind::Mul(factors) => Expr::product(
                factors
                    .iter()
                    .map(|factor| self.factor_expr(factor, ctx, steps)),
            ),
            ExprKind::Pow { base, exp } => Expr::pow(
                self.factor_expr(base, ctx, steps),
                self.factor_expr(exp, ctx, steps),
            ),
            ExprKind::Call { function, args } => Expr::call(
                function.clone(),
                args.iter().map(|arg| self.factor_expr(arg, ctx, steps)),
            ),
            ExprKind::Derivative(derivative) => Expr::derivative(
                self.factor_expr(&derivative.expr, ctx, steps),
                derivative.variable.clone(),
            ),
            ExprKind::Integral(integral) => Expr::integral(
                self.factor_expr(&integral.expr, ctx, steps),
                integral.variable.clone(),
            ),
        }
    }

    fn factor_node_to_fixpoint(
        &self,
        expr: Expr,
        ctx: &EngineContext,
        steps: &mut Vec<String>,
    ) -> Expr {
        let mut current = expr;
        let mut seen = BTreeSet::from([current.clone()]);

        while let Some(result) = self.try_factor_once(&current, ctx) {
            steps.extend(result.steps);

            let next = self.factor_children(&result.expr, ctx, steps);
            if !seen.insert(next.clone()) {
                return next;
            }

            current = next;
        }

        current
    }

    fn try_factor_once(&self, expr: &Expr, ctx: &EngineContext) -> Option<FactorizationResult> {
        for strategy in &self.strategies {
            if let Some(mut result) = strategy.try_factor(expr, ctx) {
                if result.expr == *expr {
                    continue;
                }

                if ctx.record_steps {
                    result.steps.insert(0, strategy.name().to_string());
                }
                return Some(result);
            }
        }

        None
    }
}

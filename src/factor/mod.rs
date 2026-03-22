use crate::{Expr, context::EngineContext};

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

#[derive(Default)]
pub struct Factorizer {
    strategies: Vec<Box<dyn FactorizationStrategy>>,
}

impl Factorizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_strategy(mut self, strategy: impl FactorizationStrategy + 'static) -> Self {
        self.push_strategy(strategy);
        self
    }

    pub fn push_strategy(&mut self, strategy: impl FactorizationStrategy + 'static) {
        self.strategies.push(Box::new(strategy));
    }

    pub fn factor(&self, expr: &Expr, ctx: &EngineContext) -> FactorizationResult {
        for strategy in &self.strategies {
            if let Some(mut result) = strategy.try_factor(expr, ctx) {
                if ctx.record_steps {
                    result.steps.insert(0, strategy.name().to_string());
                }
                return result;
            }
        }

        FactorizationResult::unchanged(expr.clone())
    }
}

use crate::{Expr, Symbol, context::EngineContext};

pub trait IntegrationStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn try_integrate(
        &self,
        expr: &Expr,
        variable: &Symbol,
        ctx: &EngineContext,
    ) -> Option<IntegrationResult>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntegrationStatus {
    Solved,
    Deferred,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationResult {
    pub expr: Expr,
    pub status: IntegrationStatus,
    pub steps: Vec<String>,
}

impl IntegrationResult {
    pub fn solved(expr: Expr, step: impl Into<String>) -> Self {
        Self {
            expr,
            status: IntegrationStatus::Solved,
            steps: vec![step.into()],
        }
    }

    pub fn deferred(expr: Expr) -> Self {
        Self {
            expr,
            status: IntegrationStatus::Deferred,
            steps: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct Integrator {
    strategies: Vec<Box<dyn IntegrationStrategy>>,
}

impl Integrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_strategy(mut self, strategy: impl IntegrationStrategy + 'static) -> Self {
        self.push_strategy(strategy);
        self
    }

    pub fn push_strategy(&mut self, strategy: impl IntegrationStrategy + 'static) {
        self.strategies.push(Box::new(strategy));
    }

    pub fn integrate(
        &self,
        expr: &Expr,
        variable: &Symbol,
        ctx: &EngineContext,
    ) -> IntegrationResult {
        for strategy in &self.strategies {
            if let Some(mut result) = strategy.try_integrate(expr, variable, ctx) {
                if ctx.record_steps {
                    result.steps.insert(0, strategy.name().to_string());
                }
                return result;
            }
        }

        IntegrationResult::deferred(Expr::integral(expr.clone(), variable.clone()))
    }
}

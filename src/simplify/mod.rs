use crate::{Expr, context::EngineContext, rewrite::RewriteEngine};

#[derive(Default)]
pub struct Simplifier {
    engine: RewriteEngine,
}

impl Simplifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn engine(&self) -> &RewriteEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut RewriteEngine {
        &mut self.engine
    }

    pub fn simplify(&self, expr: Expr, ctx: &EngineContext) -> Expr {
        self.engine.run(expr, ctx)
    }
}

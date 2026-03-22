mod rules;

use crate::{Expr, context::EngineContext, rewrite::RewriteEngine};

use self::rules::install_default_rules;

pub struct Simplifier {
    engine: RewriteEngine,
}

impl Default for Simplifier {
    fn default() -> Self {
        let mut engine = RewriteEngine::new();
        install_default_rules(&mut engine);
        Self { engine }
    }
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

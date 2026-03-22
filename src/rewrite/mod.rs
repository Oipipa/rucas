use crate::{
    Expr,
    context::EngineContext,
    core::visit::{Folder, fold_expr},
};

pub trait RewriteRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, expr: &Expr, ctx: &EngineContext) -> Option<Expr>;
}

#[derive(Default)]
pub struct RewriteEngine {
    rules: Vec<Box<dyn RewriteRule>>,
}

impl RewriteEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rule(mut self, rule: impl RewriteRule + 'static) -> Self {
        self.push_rule(rule);
        self
    }

    pub fn push_rule(&mut self, rule: impl RewriteRule + 'static) {
        self.rules.push(Box::new(rule));
    }

    pub fn run(&self, expr: Expr, ctx: &EngineContext) -> Expr {
        let mut current = expr;

        for _ in 0..ctx.rewrite_budget {
            let next = self.run_pass(&current, ctx);
            if next == current {
                return current;
            }
            current = next;
        }

        current
    }

    fn run_pass(&self, expr: &Expr, ctx: &EngineContext) -> Expr {
        let mut folder = RuleFolder {
            rules: &self.rules,
            ctx,
        };
        fold_expr(&mut folder, expr)
    }
}

struct RuleFolder<'a> {
    rules: &'a [Box<dyn RewriteRule>],
    ctx: &'a EngineContext,
}

impl Folder for RuleFolder<'_> {
    fn rewrite(&mut self, expr: Expr) -> Expr {
        for rule in self.rules {
            if let Some(next) = rule.apply(&expr, self.ctx) {
                return next;
            }
        }

        expr
    }
}

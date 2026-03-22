#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineContext {
    pub rewrite_budget: usize,
    pub auto_simplify: bool,
    pub record_steps: bool,
}

impl Default for EngineContext {
    fn default() -> Self {
        Self {
            rewrite_budget: 8,
            auto_simplify: true,
            record_steps: true,
        }
    }
}

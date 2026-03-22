use rucas::{
    EngineContext, Expr, FactorizationResult, FactorizationStatus, FactorizationStrategy,
    Factorizer,
};

struct NeverApplies;

impl FactorizationStrategy for NeverApplies {
    fn name(&self) -> &'static str {
        "never"
    }

    fn try_factor(&self, _expr: &Expr, _ctx: &EngineContext) -> Option<FactorizationResult> {
        None
    }
}

struct DemoFactor;

impl FactorizationStrategy for DemoFactor {
    fn name(&self) -> &'static str {
        "demo-factor"
    }

    fn try_factor(&self, expr: &Expr, _ctx: &EngineContext) -> Option<FactorizationResult> {
        let target = Expr::sum([
            Expr::product([Expr::symbol("x"), Expr::symbol("y")]),
            Expr::symbol("x"),
        ]);

        if expr == &target {
            Some(FactorizationResult::factored(
                Expr::product([
                    Expr::symbol("x"),
                    Expr::sum([Expr::symbol("y"), Expr::integer(1)]),
                ]),
                "common factor extracted",
            ))
        } else {
            None
        }
    }
}

#[test]
fn factorizer_uses_first_matching_strategy_and_records_steps() {
    let ctx = EngineContext::default();
    let expr = Expr::sum([
        Expr::product([Expr::symbol("x"), Expr::symbol("y")]),
        Expr::symbol("x"),
    ]);

    let result = Factorizer::new()
        .with_strategy(NeverApplies)
        .with_strategy(DemoFactor)
        .factor(&expr, &ctx);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(result.expr.to_string(), "x * (1 + y)");
    assert_eq!(result.steps, vec!["demo-factor", "common factor extracted"]);
}

#[test]
fn factorizer_can_skip_step_recording() {
    let ctx = EngineContext {
        record_steps: false,
        ..EngineContext::default()
    };
    let expr = Expr::sum([
        Expr::product([Expr::symbol("x"), Expr::symbol("y")]),
        Expr::symbol("x"),
    ]);

    let result = Factorizer::new()
        .with_strategy(DemoFactor)
        .factor(&expr, &ctx);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(result.steps, vec!["common factor extracted"]);
}

use rucas::{
    BuiltinFunction, Differentiator, EngineContext, Expr, FactorizationStatus, Factorizer,
    Function, IntegrationStatus, Integrator, Symbol,
};

#[test]
fn addition_is_canonicalized() {
    let expr = Expr::sum([
        Expr::symbol("y"),
        Expr::integer(0),
        Expr::symbol("x"),
        Expr::integer(2),
    ]);

    assert_eq!(expr.to_string(), "2 + x + y");
}

#[test]
fn unsupported_derivatives_remain_explicit() {
    let ctx = EngineContext::default();
    let x = Symbol::new("x");
    let expr = Expr::call(
        Function::Builtin(BuiltinFunction::Sin),
        [Expr::from_symbol(x.clone())],
    );

    let derivative = Differentiator::default().differentiate(&expr, &x, &ctx);

    assert_eq!(derivative.to_string(), "d/dx(sin(x))");
}

#[test]
fn integration_defaults_to_unevaluated_nodes() {
    let ctx = EngineContext::default();
    let x = Symbol::new("x");
    let expr = Expr::from_symbol(x.clone());

    let result = Integrator::default().integrate(&expr, &x, &ctx);

    assert_eq!(result.status, IntegrationStatus::Deferred);
    assert_eq!(result.expr.to_string(), "int(x) dx");
}

#[test]
fn factorization_defaults_to_unchanged_expression() {
    let ctx = EngineContext::default();
    let expr = Expr::sum([Expr::symbol("x"), Expr::integer(1)]);

    let result = Factorizer::default().factor(&expr, &ctx);

    assert_eq!(result.status, FactorizationStatus::Unchanged);
    assert_eq!(result.expr, expr);
    assert!(result.steps.is_empty());
}

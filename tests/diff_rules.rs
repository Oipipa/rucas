use rucas::{BuiltinFunction, Differentiator, EngineContext, Expr, Function, Symbol};

fn differentiate(expr: &Expr, variable: &Symbol) -> Expr {
    Differentiator::new().differentiate(expr, variable, &EngineContext::default())
}

fn differentiate_raw(expr: &Expr, variable: &Symbol) -> Expr {
    Differentiator::new().differentiate(
        expr,
        variable,
        &EngineContext {
            auto_simplify: false,
            ..EngineContext::default()
        },
    )
}

#[test]
fn constants_and_symbols_differentiate() {
    let x = Symbol::new("x");
    let y = Symbol::new("y");

    assert_eq!(differentiate(&Expr::integer(7), &x), Expr::zero());
    assert_eq!(
        differentiate(&Expr::from_symbol(x.clone()), &x),
        Expr::one()
    );
    assert_eq!(
        differentiate(&Expr::from_symbol(y.clone()), &x),
        Expr::zero()
    );
}

#[test]
fn products_apply_the_product_rule() {
    let x = Symbol::new("x");
    let expr = Expr::product([Expr::from_symbol(x.clone()), Expr::from_symbol(x.clone())]);

    let derivative = differentiate(&expr, &x);
    let expected = Expr::product([Expr::integer(2), Expr::from_symbol(x.clone())]);

    assert_eq!(derivative, expected);
}

#[test]
fn powers_with_constant_exponents_differentiate() {
    let x = Symbol::new("x");
    let y = Symbol::new("y");

    let cubic = Expr::pow(Expr::from_symbol(x.clone()), Expr::integer(3));
    assert_eq!(
        differentiate(&cubic, &x),
        Expr::product([
            Expr::integer(3),
            Expr::pow(Expr::from_symbol(x.clone()), Expr::integer(2)),
        ])
    );

    let symbolic_exponent = Expr::pow(Expr::from_symbol(x.clone()), Expr::from_symbol(y.clone()));
    assert_eq!(
        differentiate(&symbolic_exponent, &x),
        Expr::product([
            Expr::from_symbol(y.clone()),
            Expr::pow(
                Expr::from_symbol(x.clone()),
                Expr::sum([Expr::integer(-1), Expr::from_symbol(y.clone())]),
            ),
        ])
    );
}

#[test]
fn constant_base_exponentials_differentiate_via_log() {
    let x = Symbol::new("x");
    let expr = Expr::pow(Expr::integer(2), Expr::from_symbol(x.clone()));

    assert_eq!(
        differentiate(&expr, &x),
        Expr::product([
            expr.clone(),
            Expr::call(Function::Builtin(BuiltinFunction::Log), [Expr::integer(2)]),
        ])
    );
}

#[test]
fn general_powers_use_logarithmic_differentiation() {
    let x = Symbol::new("x");
    let expr = Expr::pow(Expr::from_symbol(x.clone()), Expr::from_symbol(x.clone()));

    assert_eq!(
        differentiate(&expr, &x),
        Expr::product([
            expr.clone(),
            Expr::sum([
                Expr::integer(1),
                Expr::call(
                    Function::Builtin(BuiltinFunction::Log),
                    [Expr::from_symbol(x.clone())],
                ),
            ]),
        ])
    );
}

#[test]
fn builtin_functions_apply_direct_rules_and_chain_rule() {
    let x = Symbol::new("x");

    let sin_expr = Expr::call(
        Function::Builtin(BuiltinFunction::Sin),
        [Expr::pow(Expr::from_symbol(x.clone()), Expr::integer(2))],
    );
    assert_eq!(
        differentiate(&sin_expr, &x),
        Expr::product([
            Expr::integer(2),
            Expr::from_symbol(x.clone()),
            Expr::call(
                Function::Builtin(BuiltinFunction::Cos),
                [Expr::pow(Expr::from_symbol(x.clone()), Expr::integer(2))],
            ),
        ])
    );

    let cos_expr = Expr::call(
        Function::Builtin(BuiltinFunction::Cos),
        [Expr::from_symbol(x.clone())],
    );
    assert_eq!(
        differentiate(&cos_expr, &x),
        Expr::product([
            Expr::integer(-1),
            Expr::call(
                Function::Builtin(BuiltinFunction::Sin),
                [Expr::from_symbol(x.clone())],
            ),
        ])
    );

    let exp_expr = Expr::call(
        Function::Builtin(BuiltinFunction::Exp),
        [Expr::from_symbol(x.clone())],
    );
    assert_eq!(differentiate(&exp_expr, &x), exp_expr);

    let log_expr = Expr::call(
        Function::Builtin(BuiltinFunction::Log),
        [Expr::from_symbol(x.clone())],
    );
    assert_eq!(
        differentiate(&log_expr, &x),
        Expr::pow(Expr::from_symbol(x.clone()), Expr::integer(-1))
    );

    let nested_exp = Expr::call(
        Function::Builtin(BuiltinFunction::Exp),
        [Expr::pow(Expr::from_symbol(x.clone()), Expr::integer(2))],
    );
    assert_eq!(
        differentiate(&nested_exp, &x),
        Expr::product([Expr::integer(2), Expr::from_symbol(x.clone()), nested_exp,])
    );
}

#[test]
fn unsupported_calls_remain_explicit_inside_supported_rules() {
    let x = Symbol::new("x");
    let call = Expr::call(Function::named("f"), [Expr::from_symbol(x.clone())]);
    let expr = Expr::product([Expr::from_symbol(x.clone()), call.clone()]);

    let derivative = differentiate(&expr, &x);
    let expected = Expr::sum([
        call.clone(),
        Expr::product([
            Expr::from_symbol(x.clone()),
            Expr::derivative(call.clone(), x.clone()),
        ]),
    ]);

    assert_eq!(derivative, expected);
}

#[test]
fn zero_base_with_variable_exponent_remains_explicit() {
    let x = Symbol::new("x");
    let expr = Expr::pow(Expr::zero(), Expr::from_symbol(x.clone()));

    assert_eq!(
        differentiate(&expr, &x),
        Expr::derivative(expr.clone(), x.clone())
    );
}

#[test]
fn differentiator_can_skip_the_simplify_pipeline() {
    let x = Symbol::new("x");
    let expr = Expr::product([Expr::from_symbol(x.clone()), Expr::from_symbol(x.clone())]);

    assert_eq!(
        differentiate_raw(&expr, &x),
        Expr::sum([Expr::from_symbol(x.clone()), Expr::from_symbol(x.clone())])
    );
}

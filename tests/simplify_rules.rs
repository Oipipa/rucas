use rucas::{BuiltinFunction, EngineContext, Expr, Function, Simplifier};

fn simplify(expr: Expr) -> Expr {
    Simplifier::new().simplify(expr, &EngineContext::default())
}

#[test]
fn additive_polynomials_are_collected_through_nested_products() {
    let x = Expr::symbol("x");
    let expr = Expr::sum([
        Expr::product([x.clone(), Expr::sum([Expr::one(), x.clone()])]),
        Expr::pow(x.clone(), Expr::integer(2)),
    ]);

    assert_eq!(
        simplify(expr),
        Expr::sum([
            Expr::symbol("x"),
            Expr::product([
                Expr::integer(2),
                Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            ]),
        ])
    );
}

#[test]
fn additive_polynomials_are_collected_through_supported_powers() {
    let x = Expr::symbol("x");
    let expr = Expr::sum([
        Expr::pow(Expr::sum([Expr::one(), x.clone()]), Expr::integer(2)),
        x,
    ]);

    assert_eq!(
        simplify(expr),
        Expr::sum([
            Expr::integer(1),
            Expr::product([Expr::integer(3), Expr::symbol("x")]),
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        ])
    );
}

#[test]
fn standalone_products_are_not_expanded_by_default() {
    let expr = Expr::product([
        Expr::sum([Expr::one(), Expr::symbol("x")]),
        Expr::sum([Expr::integer(2), Expr::symbol("x")]),
    ]);

    assert_eq!(simplify(expr.clone()), expr);
}

#[test]
fn polynomial_rational_products_cancel_common_factors() {
    let expr = Expr::product([
        Expr::sum([
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            Expr::integer(-1),
        ]),
        Expr::pow(
            Expr::sum([Expr::integer(-1), Expr::symbol("x")]),
            Expr::integer(-1),
        ),
    ]);

    assert_eq!(
        simplify(expr),
        Expr::sum([Expr::integer(1), Expr::symbol("x")])
    );
}

#[test]
fn polynomial_rational_products_normalize_denominator_scale() {
    let expr = Expr::product([
        Expr::integer(2),
        Expr::pow(
            Expr::sum([
                Expr::integer(2),
                Expr::product([Expr::integer(2), Expr::symbol("x")]),
            ]),
            Expr::integer(-1),
        ),
    ]);

    assert_eq!(
        simplify(expr),
        Expr::pow(
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
            Expr::integer(-1),
        )
    );
}

#[test]
fn polynomial_rational_sums_collect_common_denominators() {
    let expr = Expr::sum([
        Expr::pow(
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
            Expr::integer(-1),
        ),
        Expr::product([
            Expr::symbol("x"),
            Expr::pow(
                Expr::sum([Expr::integer(1), Expr::symbol("x")]),
                Expr::integer(-1),
            ),
        ]),
    ]);

    assert_eq!(simplify(expr), Expr::integer(1));
}

#[test]
fn polynomial_rational_sums_combine_distinct_denominators() {
    let expr = Expr::sum([
        Expr::pow(
            Expr::sum([Expr::integer(-1), Expr::symbol("x")]),
            Expr::integer(-1),
        ),
        Expr::pow(
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
            Expr::integer(-1),
        ),
    ]);

    assert_eq!(
        simplify(expr),
        Expr::product([
            Expr::product([Expr::integer(2), Expr::symbol("x")]),
            Expr::pow(
                Expr::sum([
                    Expr::integer(-1),
                    Expr::pow(Expr::symbol("x"), Expr::integer(2)),
                ]),
                Expr::integer(-1),
            ),
        ])
    );
}

#[test]
fn like_terms_are_collected_with_exact_coefficients() {
    let expr = Expr::sum([
        Expr::symbol("x"),
        Expr::product([Expr::integer(2), Expr::symbol("x")]),
        Expr::product([Expr::integer(-3), Expr::symbol("x")]),
        Expr::symbol("y"),
    ]);

    assert_eq!(simplify(expr), Expr::symbol("y"));
}

#[test]
fn repeated_factors_are_combined_into_powers() {
    let expr = Expr::product([
        Expr::symbol("x"),
        Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        Expr::pow(Expr::symbol("x"), Expr::integer(-1)),
        Expr::symbol("y"),
    ]);

    assert_eq!(
        simplify(expr),
        Expr::product([
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            Expr::symbol("y"),
        ])
    );
}

#[test]
fn non_integer_power_products_remain_explicit() {
    let expr = Expr::product([
        Expr::pow(Expr::symbol("x"), Expr::rational(1, 2)),
        Expr::pow(Expr::symbol("x"), Expr::rational(1, 2)),
    ]);

    assert_eq!(simplify(expr.clone()), expr);
}

#[test]
fn builtin_exact_identities_are_simplified() {
    assert_eq!(
        simplify(Expr::call(
            Function::Builtin(BuiltinFunction::Sin),
            [Expr::zero()],
        )),
        Expr::zero()
    );
    assert_eq!(
        simplify(Expr::call(
            Function::Builtin(BuiltinFunction::Cos),
            [Expr::zero()],
        )),
        Expr::one()
    );
    assert_eq!(
        simplify(Expr::call(
            Function::Builtin(BuiltinFunction::Exp),
            [Expr::zero()],
        )),
        Expr::one()
    );
    assert_eq!(
        simplify(Expr::call(
            Function::Builtin(BuiltinFunction::Log),
            [Expr::one()],
        )),
        Expr::zero()
    );
}

#[test]
fn builtin_function_rules_cover_parity_and_exponentials() {
    let neg_x = Expr::product([Expr::integer(-1), Expr::symbol("x")]);

    assert_eq!(
        simplify(Expr::call(
            Function::Builtin(BuiltinFunction::Sin),
            [neg_x.clone()],
        )),
        Expr::product([
            Expr::integer(-1),
            Expr::call(Function::Builtin(BuiltinFunction::Sin), [Expr::symbol("x")]),
        ])
    );

    assert_eq!(
        simplify(Expr::call(
            Function::Builtin(BuiltinFunction::Cos),
            [neg_x.clone()],
        )),
        Expr::call(Function::Builtin(BuiltinFunction::Cos), [Expr::symbol("x")])
    );

    assert_eq!(
        simplify(Expr::pow(
            Expr::call(Function::Builtin(BuiltinFunction::Exp), [Expr::symbol("x")]),
            Expr::integer(3),
        )),
        Expr::call(
            Function::Builtin(BuiltinFunction::Exp),
            [Expr::product([Expr::integer(3), Expr::symbol("x")])],
        )
    );

    assert_eq!(
        simplify(Expr::product([
            Expr::call(Function::Builtin(BuiltinFunction::Exp), [Expr::symbol("x")]),
            Expr::call(Function::Builtin(BuiltinFunction::Exp), [Expr::symbol("y")]),
        ])),
        Expr::call(
            Function::Builtin(BuiltinFunction::Exp),
            [Expr::sum([Expr::symbol("x"), Expr::symbol("y")])],
        )
    );
}

#[test]
fn simplify_runs_to_fixpoint_across_rule_boundaries() {
    let expr = Expr::sum([
        Expr::product([
            Expr::symbol("x"),
            Expr::pow(Expr::symbol("x"), Expr::integer(-1)),
        ]),
        Expr::product([
            Expr::integer(2),
            Expr::symbol("x"),
            Expr::pow(Expr::symbol("x"), Expr::integer(-1)),
        ]),
    ]);

    assert_eq!(simplify(expr), Expr::integer(3));
}

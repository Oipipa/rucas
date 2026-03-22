use rucas::{EngineContext, Expr, FactorizationStatus, Factorizer};

fn factor(expr: &Expr) -> rucas::FactorizationResult {
    Factorizer::new().factor(expr, &EngineContext::default())
}

#[test]
fn default_factorizer_extracts_numeric_and_symbolic_content() {
    let expr = Expr::sum([
        Expr::product([Expr::integer(2), Expr::symbol("x")]),
        Expr::product([Expr::integer(4), Expr::symbol("x"), Expr::symbol("y")]),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::integer(2),
            Expr::symbol("x"),
            Expr::sum([
                Expr::integer(1),
                Expr::product([Expr::integer(2), Expr::symbol("y")]),
            ]),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["common-factor", "common factor extracted"]
    );
}

#[test]
fn factorizer_extracts_positive_integer_power_content() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        Expr::symbol("x"),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::symbol("x"),
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
        ])
    );
}

#[test]
fn polynomial_factorizer_handles_repeated_quadratic_roots() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        Expr::product([Expr::integer(2), Expr::symbol("x")]),
        Expr::integer(1),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::pow(
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
            Expr::integer(2),
        )
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "rational root factorization"]
    );
}

#[test]
fn polynomial_factorizer_handles_non_monic_quadratics() {
    let expr = Expr::sum([
        Expr::product([
            Expr::integer(2),
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        ]),
        Expr::product([Expr::integer(5), Expr::symbol("x")]),
        Expr::integer(2),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::sum([
                Expr::integer(1),
                Expr::product([Expr::integer(2), Expr::symbol("x")])
            ]),
            Expr::sum([Expr::integer(2), Expr::symbol("x")]),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "rational root factorization"]
    );
}

#[test]
fn polynomial_factorizer_handles_unexpanded_quadratics() {
    let x = Expr::symbol("x");
    let expr = Expr::sum([
        Expr::product([x.clone(), Expr::sum([Expr::one(), x.clone()])]),
        x,
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::symbol("x"),
            Expr::sum([Expr::integer(2), Expr::symbol("x")]),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "rational root factorization"]
    );
}

#[test]
fn polynomial_factorizer_handles_cubics_with_multiple_rational_roots() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(3)),
        Expr::product([Expr::integer(-1), Expr::symbol("x")]),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::symbol("x"),
            Expr::sum([Expr::integer(-1), Expr::symbol("x")]),
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "rational root factorization"]
    );
}

#[test]
fn polynomial_factorizer_handles_repeated_cubic_roots() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(3)),
        Expr::product([
            Expr::integer(3),
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        ]),
        Expr::product([Expr::integer(3), Expr::symbol("x")]),
        Expr::integer(1),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::pow(
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
            Expr::integer(3),
        )
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "rational root factorization"]
    );
}

#[test]
fn polynomial_factorizer_carries_leading_scale_through_root_extraction() {
    let expr = Expr::sum([
        Expr::product([
            Expr::integer(2),
            Expr::pow(Expr::symbol("x"), Expr::integer(3)),
        ]),
        Expr::product([Expr::integer(-2), Expr::symbol("x")]),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::integer(2),
            Expr::symbol("x"),
            Expr::sum([Expr::integer(-1), Expr::symbol("x")]),
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "rational root factorization"]
    );
}

#[test]
fn polynomial_factorizer_handles_difference_of_squares_without_rational_roots() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(4)),
        Expr::integer(-4),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::sum([
                Expr::pow(Expr::symbol("x"), Expr::integer(2)),
                Expr::integer(-2),
            ]),
            Expr::sum([
                Expr::pow(Expr::symbol("x"), Expr::integer(2)),
                Expr::integer(2),
            ]),
        ])
    );
    assert_eq!(result.steps, vec!["polynomial", "difference of squares"]);
}

#[test]
fn polynomial_factorizer_handles_power_substitution_for_even_powers() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(4)),
        Expr::product([
            Expr::integer(3),
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        ]),
        Expr::integer(2),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::sum([
                Expr::integer(1),
                Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            ]),
            Expr::sum([
                Expr::integer(2),
                Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            ]),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "power substitution factorization"]
    );
}

#[test]
fn polynomial_factorizer_handles_power_substitution_for_higher_powers() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(6)),
        Expr::product([
            Expr::integer(5),
            Expr::pow(Expr::symbol("x"), Expr::integer(3)),
        ]),
        Expr::integer(6),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::sum([
                Expr::integer(2),
                Expr::pow(Expr::symbol("x"), Expr::integer(3)),
            ]),
            Expr::sum([
                Expr::integer(3),
                Expr::pow(Expr::symbol("x"), Expr::integer(3)),
            ]),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "power substitution factorization"]
    );
}

#[test]
fn polynomial_factorizer_recovers_repeated_irreducible_factors() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(4)),
        Expr::product([
            Expr::integer(2),
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        ]),
        Expr::integer(1),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::pow(
            Expr::sum([
                Expr::integer(1),
                Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            ]),
            Expr::integer(2),
        )
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "square-free factorization"]
    );
}

#[test]
fn factorizer_composes_common_factor_and_polynomial_factorization() {
    let expr = Expr::sum([
        Expr::product([
            Expr::symbol("a"),
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        ]),
        Expr::product([Expr::integer(2), Expr::symbol("a"), Expr::symbol("x")]),
        Expr::symbol("a"),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::symbol("a"),
            Expr::pow(
                Expr::sum([Expr::integer(1), Expr::symbol("x")]),
                Expr::integer(2),
            ),
        ])
    );
    assert_eq!(
        result.steps,
        vec![
            "common-factor",
            "common factor extracted",
            "polynomial",
            "rational root factorization",
        ]
    );
}

#[test]
fn factorizer_factors_nested_subexpressions() {
    let expr = Expr::product([
        Expr::symbol("y"),
        Expr::sum([
            Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            Expr::integer(-1),
        ]),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([
            Expr::sum([Expr::integer(-1), Expr::symbol("x")]),
            Expr::sum([Expr::integer(1), Expr::symbol("x")]),
            Expr::symbol("y"),
        ])
    );
    assert_eq!(
        result.steps,
        vec!["polynomial", "rational root factorization"]
    );
}

#[test]
fn factorizer_leaves_unrelated_sums_unchanged() {
    let expr = Expr::sum([Expr::symbol("x"), Expr::symbol("y")]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Unchanged);
    assert_eq!(result.expr, expr);
}

#[test]
fn polynomial_factorizer_leaves_irreducible_quadratics_unchanged() {
    let expr = Expr::sum([
        Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        Expr::integer(1),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Unchanged);
    assert_eq!(result.expr, expr);
}

#[test]
fn factorizer_can_factor_exact_non_integer_atomic_factors_without_splitting_them() {
    let root = Expr::pow(Expr::symbol("x"), Expr::rational(1, 2));
    let expr = Expr::sum([
        root.clone(),
        Expr::product([root.clone(), Expr::symbol("y")]),
    ]);

    let result = factor(&expr);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(
        result.expr,
        Expr::product([root, Expr::sum([Expr::integer(1), Expr::symbol("y")]),])
    );
}

#[test]
fn factorizer_skips_step_recording_when_requested() {
    let ctx = EngineContext {
        record_steps: false,
        ..EngineContext::default()
    };
    let expr = Expr::sum([
        Expr::product([Expr::integer(2), Expr::symbol("x")]),
        Expr::product([Expr::integer(4), Expr::symbol("x"), Expr::symbol("y")]),
    ]);

    let result = Factorizer::new().factor(&expr, &ctx);

    assert_eq!(result.status, FactorizationStatus::Factored);
    assert_eq!(result.steps, vec!["common factor extracted"]);
}

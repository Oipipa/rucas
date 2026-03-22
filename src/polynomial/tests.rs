use std::collections::BTreeMap;

use num_bigint::BigInt;

use super::{PolynomialAnalysisLimits, UnivariatePolynomial, UnivariateRationalFunction};
use crate::{Expr, Number, Symbol};

#[test]
fn recursive_analysis_collects_nested_polynomials() {
    let x = Expr::symbol("x");
    let expr = Expr::sum([
        Expr::product([x.clone(), Expr::sum([Expr::one(), x.clone()])]),
        Expr::pow(Expr::sum([Expr::one(), x.clone()]), Expr::integer(2)),
    ]);

    let polynomial = UnivariatePolynomial::from_expr(&expr)
        .expect("supported univariate polynomial should analyze");

    assert_eq!(
        polynomial.to_expr(),
        Expr::sum([
            Expr::integer(1),
            Expr::product([Expr::integer(3), Expr::symbol("x")]),
            Expr::product([
                Expr::integer(2),
                Expr::pow(Expr::symbol("x"), Expr::integer(2)),
            ]),
        ])
    );
}

#[test]
fn recursive_analysis_respects_term_limits() {
    let expr = Expr::pow(
        Expr::sum([Expr::one(), Expr::symbol("x")]),
        Expr::integer(5),
    );

    let polynomial = UnivariatePolynomial::from_expr_with_limits(
        &expr,
        PolynomialAnalysisLimits { max_terms: 4 },
    );

    assert!(polynomial.is_none());
}

#[test]
fn content_and_primitive_part_preserve_exact_coefficients() {
    let polynomial = UnivariatePolynomial::from_coefficients(
        Symbol::new("x"),
        [
            (0, Number::rational(2, 3)),
            (1, Number::rational(4, 3)),
            (2, Number::rational(2, 1)),
        ],
    );

    assert_eq!(polynomial.content(), Some(Number::rational(2, 3)));
    assert_eq!(
        polynomial.primitive_part(),
        Some(UnivariatePolynomial::from_coefficients(
            Symbol::new("x"),
            [
                (0, Number::integer(1)),
                (1, Number::integer(2)),
                (2, Number::integer(3)),
            ],
        ))
    );
}

#[test]
fn integer_coefficients_are_exposed_after_primitive_normalization() {
    let polynomial = UnivariatePolynomial::from_coefficients(
        Symbol::new("x"),
        [(0, Number::rational(1, 2)), (2, Number::rational(3, 2))],
    );

    let primitive = polynomial.primitive_part().expect("non-zero polynomial");
    let integer_coefficients = primitive
        .integer_coefficients()
        .expect("primitive part should have integer coefficients");

    assert_eq!(
        integer_coefficients,
        BTreeMap::from([(0, BigInt::from(1)), (2, BigInt::from(3))])
    );
}

#[test]
fn polynomial_evaluation_and_linear_deflation_are_exact() {
    let polynomial = UnivariatePolynomial::from_coefficients(
        Symbol::new("x"),
        [
            (0, Number::integer(2)),
            (1, Number::integer(-1)),
            (2, Number::integer(-2)),
            (3, Number::integer(1)),
        ],
    );

    assert_eq!(polynomial.evaluate(&Number::integer(2)), Number::zero());

    let quotient = polynomial
        .divide_by_linear_factor(&Number::integer(2))
        .expect("x - 2 should divide exactly");

    assert_eq!(
        quotient,
        UnivariatePolynomial::from_coefficients(
            Symbol::new("x"),
            [
                (0, Number::integer(-1)),
                (1, Number::integer(0)),
                (2, Number::integer(1)),
            ],
        )
    );
}

#[test]
fn repeated_linear_roots_can_be_deflated() {
    let polynomial = UnivariatePolynomial::from_expr(&Expr::pow(
        Expr::sum([Expr::symbol("x"), Expr::integer(1)]),
        Expr::integer(3),
    ))
    .expect("expanded polynomial should analyze");

    let (multiplicity, quotient) = polynomial
        .deflate_linear_root(&Number::integer(-1))
        .expect("root should deflate");

    assert_eq!(multiplicity, 3);
    assert_eq!(
        quotient,
        UnivariatePolynomial::from_coefficients(Symbol::new("x"), [(0, Number::one())])
    );
}

#[test]
fn exact_division_and_gcd_work_for_repeated_irreducible_factors() {
    let polynomial = UnivariatePolynomial::from_coefficients(
        Symbol::new("x"),
        [
            (0, Number::integer(1)),
            (2, Number::integer(2)),
            (4, Number::integer(1)),
        ],
    );
    let divisor = UnivariatePolynomial::from_coefficients(
        Symbol::new("x"),
        [(0, Number::integer(1)), (2, Number::integer(1))],
    );

    assert_eq!(polynomial.exact_div(&divisor), Some(divisor.clone()));
    assert_eq!(
        polynomial.derivative(),
        UnivariatePolynomial::from_coefficients(
            Symbol::new("x"),
            [(1, Number::integer(4)), (3, Number::integer(4)),],
        )
    );
    assert_eq!(polynomial.gcd(&polynomial.derivative()), Some(divisor));
}

#[test]
fn square_free_decomposition_tracks_factor_multiplicity() {
    let polynomial = UnivariatePolynomial::from_coefficients(
        Symbol::new("x"),
        [
            (0, Number::integer(1)),
            (2, Number::integer(2)),
            (4, Number::integer(1)),
        ],
    );

    assert_eq!(
        polynomial.square_free_decomposition(),
        Some(vec![(
            UnivariatePolynomial::from_coefficients(
                Symbol::new("x"),
                [(0, Number::integer(1)), (2, Number::integer(1))],
            ),
            2,
        )])
    );
}

#[test]
fn rational_normalization_cancels_polynomial_gcd() {
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

    let rational = UnivariateRationalFunction::from_expr(&expr)
        .expect("supported rational form should analyze");

    assert_eq!(
        rational.numerator(),
        &UnivariatePolynomial::from_coefficients(
            Symbol::new("x"),
            [(0, Number::integer(1)), (1, Number::integer(1))],
        )
    );
    assert_eq!(
        rational.denominator(),
        &UnivariatePolynomial::from_coefficients(Symbol::new("x"), [(0, Number::integer(1))])
    );
}

#[test]
fn rational_normalization_makes_denominators_monic() {
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

    let rational = UnivariateRationalFunction::from_expr(&expr)
        .expect("supported rational form should analyze");

    assert_eq!(
        rational.numerator(),
        &UnivariatePolynomial::from_coefficients(Symbol::new("x"), [(0, Number::integer(1))])
    );
    assert_eq!(
        rational.denominator(),
        &UnivariatePolynomial::from_coefficients(
            Symbol::new("x"),
            [(0, Number::integer(1)), (1, Number::integer(1))],
        )
    );
}

#[test]
fn rational_normalization_collects_additive_terms_with_common_denominator() {
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

    let rational = UnivariateRationalFunction::from_expr(&expr)
        .expect("supported additive rational form should analyze");

    assert_eq!(
        rational.numerator(),
        &UnivariatePolynomial::from_coefficients(Symbol::new("x"), [(0, Number::integer(1))])
    );
    assert_eq!(
        rational.denominator(),
        &UnivariatePolynomial::from_coefficients(Symbol::new("x"), [(0, Number::integer(1))])
    );
}

#[test]
fn rational_normalization_combines_distinct_denominators() {
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

    let rational = UnivariateRationalFunction::from_expr(&expr)
        .expect("supported additive rational form should analyze");

    assert_eq!(
        rational.numerator(),
        &UnivariatePolynomial::from_coefficients(Symbol::new("x"), [(1, Number::integer(2))])
    );
    assert_eq!(
        rational.denominator(),
        &UnivariatePolynomial::from_coefficients(
            Symbol::new("x"),
            [(0, Number::integer(-1)), (2, Number::integer(1))],
        )
    );
}

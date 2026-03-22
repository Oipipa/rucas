use num_bigint::BigInt;
use rucas::{
    Expr, Number,
    core::visit::{Visitor, contains, replace, rewrite_bottom_up, walk},
};

#[test]
fn big_numbers_normalize_exactly() {
    let big = BigInt::parse_bytes(b"123456789012345678901234567890", 10).unwrap();

    let reduced = Number::rational(big.clone() * BigInt::from(3u32), BigInt::from(3u32));
    assert_eq!(reduced.to_string(), big.to_string());

    let signed = Number::rational(-2, -4);
    assert_eq!(signed.to_string(), "1/2");

    let reciprocal = Number::integer(2).powi(-3).unwrap();
    assert_eq!(reciprocal.to_string(), "1/8");
}

#[test]
fn canonicalization_collects_exact_numeric_factors() {
    let expr = Expr::product([
        Expr::integer(4),
        Expr::symbol("y"),
        Expr::product([Expr::rational(1, 2), Expr::symbol("x")]),
        Expr::integer(1),
    ]);

    assert_eq!(expr.to_string(), "2 * x * y");
}

#[test]
fn powers_normalize_without_ambiguous_printing() {
    let cubic = Expr::pow(
        Expr::pow(Expr::symbol("x"), Expr::integer(2)),
        Expr::integer(3),
    );
    assert_eq!(cubic.to_string(), "x^6");

    let squared_root = Expr::pow(
        Expr::pow(Expr::symbol("x"), Expr::rational(1, 2)),
        Expr::integer(2),
    );
    assert_eq!(squared_root.to_string(), "x");

    let fractional_exp = Expr::pow(Expr::symbol("x"), Expr::rational(-1, 2));
    assert_eq!(fractional_exp.to_string(), "x^(-1/2)");

    let numeric = Expr::pow(Expr::integer(-2), Expr::integer(-1));
    assert_eq!(numeric.to_string(), "-1/2");

    let symbolic = Expr::pow(Expr::integer(-2), Expr::symbol("x"));
    assert_eq!(symbolic.to_string(), "(-2)^x");
}

#[test]
fn replacement_helpers_preserve_constructor_invariants() {
    let expr = Expr::product([Expr::symbol("x"), Expr::integer(1)]);
    let replacement = Expr::sum([Expr::integer(0), Expr::symbol("z")]);

    let rewritten = replace(&expr, &Expr::symbol("x"), &replacement);

    assert_eq!(rewritten.to_string(), "z");
}

#[test]
fn bottom_up_rewrites_can_rebuild_nested_expressions() {
    let expr = Expr::sum([
        Expr::symbol("x"),
        Expr::pow(Expr::symbol("y"), Expr::integer(2)),
    ]);

    let rewritten = rewrite_bottom_up(&expr, &mut |candidate| {
        if candidate == Expr::integer(2) {
            Expr::integer(3)
        } else {
            candidate
        }
    });

    assert_eq!(rewritten.to_string(), "x + y^3");
    assert!(contains(&rewritten, &Expr::integer(3)));
}

#[test]
fn walk_visits_every_supported_node() {
    #[derive(Default)]
    struct Counter {
        enters: usize,
        exits: usize,
    }

    impl Visitor for Counter {
        fn enter(&mut self, _expr: &Expr) {
            self.enters += 1;
        }

        fn exit(&mut self, _expr: &Expr) {
            self.exits += 1;
        }
    }

    let expr = Expr::sum([
        Expr::symbol("x"),
        Expr::pow(Expr::symbol("y"), Expr::integer(2)),
    ]);

    let mut counter = Counter::default();
    walk(&expr, &mut counter);

    assert_eq!(counter.enters, 5);
    assert_eq!(counter.exits, 5);
}

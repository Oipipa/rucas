use rucas::{Assumptions, Symbol};

#[test]
fn symbols_store_assumptions_without_changing_display() {
    let assumptions = Assumptions {
        real: true,
        positive: true,
        integer: true,
    };
    let symbol = Symbol::with_assumptions("n", assumptions.clone());

    assert_eq!(symbol.name(), "n");
    assert_eq!(symbol.assumptions(), &assumptions);
    assert_eq!(symbol.to_string(), "n");
}

#[test]
fn symbol_equality_and_order_are_deterministic() {
    let plain_x = Symbol::new("x");
    let assumed_x = Symbol::with_assumptions(
        "x",
        Assumptions {
            integer: true,
            ..Assumptions::default()
        },
    );
    let y = Symbol::new("y");

    assert_eq!(plain_x, Symbol::new("x"));
    assert_ne!(plain_x, assumed_x);
    assert!(plain_x < y);
}

use crate::{Expr, Number, Symbol};

use super::UnivariatePolynomial;

impl UnivariatePolynomial {
    pub(crate) fn to_expr(&self) -> Expr {
        Expr::sum(
            self.coefficients
                .iter()
                .map(|(degree, coefficient)| monomial_expr(&self.variable, *degree, coefficient)),
        )
    }
}

pub(crate) fn monomial_expr(variable: &Symbol, degree: usize, coefficient: &Number) -> Expr {
    match degree {
        0 => Expr::number(coefficient.clone()),
        1 => match coefficient {
            value if value.is_one() => Expr::from_symbol(variable.clone()),
            _ => Expr::product([
                Expr::number(coefficient.clone()),
                Expr::from_symbol(variable.clone()),
            ]),
        },
        _ => {
            let power = Expr::pow(
                Expr::from_symbol(variable.clone()),
                Expr::integer(degree as i64),
            );

            if coefficient.is_one() {
                power
            } else {
                Expr::product([Expr::number(coefficient.clone()), power])
            }
        }
    }
}

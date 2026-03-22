use crate::{Expr, Number, Symbol};

use super::{PolynomialCoefficient, SparseUnivariatePolynomial};

impl SparseUnivariatePolynomial<Number> {
    pub(crate) fn to_expr(&self) -> Expr {
        polynomial_expr(self, |coefficient| Expr::number(coefficient.clone()))
    }
}

impl SparseUnivariatePolynomial<Expr> {
    pub(crate) fn to_expr(&self) -> Expr {
        polynomial_expr(self, Clone::clone)
    }
}

pub(crate) fn monomial_expr(variable: &Symbol, degree: usize, coefficient: &Number) -> Expr {
    monomial_expr_with_coefficient(variable, degree, Expr::number(coefficient.clone()))
}

pub(super) fn monomial_expr_with_coefficient(
    variable: &Symbol,
    degree: usize,
    coefficient: Expr,
) -> Expr {
    match degree {
        0 => coefficient,
        1 => match coefficient {
            value if value.is_one() => Expr::from_symbol(variable.clone()),
            _ => Expr::product([coefficient, Expr::from_symbol(variable.clone())]),
        },
        _ => {
            let power = Expr::pow(
                Expr::from_symbol(variable.clone()),
                Expr::integer(degree as i64),
            );

            if coefficient.is_one() {
                power
            } else {
                Expr::product([coefficient, power])
            }
        }
    }
}

fn polynomial_expr<C>(
    polynomial: &SparseUnivariatePolynomial<C>,
    coefficient_expr: impl Fn(&C) -> Expr,
) -> Expr
where
    C: PolynomialCoefficient,
{
    Expr::sum(
        polynomial
            .coefficients()
            .iter()
            .map(|(degree, coefficient)| {
                monomial_expr_with_coefficient(
                    polynomial.variable(),
                    *degree,
                    coefficient_expr(coefficient),
                )
            }),
    )
}

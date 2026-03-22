use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::Signed;

use crate::{Expr, ExprKind, Number};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExactProduct {
    coefficient: Number,
    factors: BTreeMap<Expr, usize>,
}

impl ExactProduct {
    pub(crate) fn from_expr(expr: &Expr) -> Self {
        match expr.kind() {
            ExprKind::Number(number) => Self {
                coefficient: number.clone(),
                factors: BTreeMap::new(),
            },
            ExprKind::Mul(factors) => {
                let mut coefficient = Number::one();
                let mut symbolic = BTreeMap::new();

                for factor in factors {
                    if let Some(number) = factor.as_number() {
                        coefficient = coefficient.mul(number);
                    } else {
                        record_factor(&mut symbolic, factor);
                    }
                }

                Self {
                    coefficient,
                    factors: symbolic,
                }
            }
            _ => {
                let mut factors = BTreeMap::new();
                record_factor(&mut factors, expr);
                Self {
                    coefficient: Number::one(),
                    factors,
                }
            }
        }
    }

    pub(crate) fn coefficient(&self) -> &Number {
        &self.coefficient
    }

    pub(crate) fn factors(&self) -> &BTreeMap<Expr, usize> {
        &self.factors
    }

    pub(crate) fn divide_by(&self, divisor: &Self) -> Option<Self> {
        if divisor.coefficient.is_zero() {
            return None;
        }

        let mut factors = self.factors.clone();

        for (factor, count) in &divisor.factors {
            let current = factors.get_mut(factor)?;
            if *current < *count {
                return None;
            }

            *current -= count;
            if *current == 0 {
                factors.remove(factor);
            }
        }

        Some(Self {
            coefficient: self.coefficient.div(&divisor.coefficient),
            factors,
        })
    }

    pub(crate) fn to_expr(&self) -> Expr {
        let mut rebuilt = Vec::new();

        if !self.coefficient.is_one() || self.factors.is_empty() {
            rebuilt.push(Expr::number(self.coefficient.clone()));
        }
        rebuilt.extend(rebuild_factors(&self.factors));

        Expr::product(rebuilt)
    }
}

pub(crate) fn common_product_factor<'a>(
    products: impl IntoIterator<Item = &'a ExactProduct>,
) -> Option<ExactProduct> {
    let mut numer_gcd: Option<BigInt> = None;
    let mut denom_lcm: Option<BigInt> = None;
    let mut all_negative = true;
    let mut saw_any = false;
    let mut common_factors = Option::<BTreeMap<Expr, usize>>::None;

    for product in products {
        saw_any = true;
        all_negative &= product.coefficient.is_negative();

        let rational = to_positive_rational(product.coefficient());
        numer_gcd = Some(match numer_gcd {
            Some(current) => current.gcd(rational.numer()),
            None => rational.numer().clone(),
        });
        denom_lcm = Some(match denom_lcm {
            Some(current) => current.lcm(rational.denom()),
            None => rational.denom().clone(),
        });

        match &mut common_factors {
            Some(common) => retain_common_factors(common, product.factors()),
            None => common_factors = Some(product.factors().clone()),
        }
    }

    if !saw_any {
        return None;
    }

    let mut coefficient = Number::from_big_rational(BigRational::new(
        numer_gcd.expect("at least one product coefficient"),
        denom_lcm.expect("at least one product coefficient"),
    ));

    if all_negative {
        coefficient = Number::from_big_rational(-to_positive_rational(&coefficient));
    }

    Some(ExactProduct {
        coefficient,
        factors: common_factors.unwrap_or_default(),
    })
}

pub(crate) fn positive_integer_power(factor: &Expr) -> Option<(Expr, usize)> {
    let ExprKind::Pow { base, exp } = factor.kind() else {
        return None;
    };

    let value = exp.as_number()?.as_i64()?;
    if value <= 1 {
        return None;
    }

    Some((base.clone(), value as usize))
}

pub(crate) fn rebuild_factors(factors: &BTreeMap<Expr, usize>) -> Vec<Expr> {
    factors
        .iter()
        .map(|(factor, count)| {
            if *count == 1 {
                factor.clone()
            } else {
                Expr::pow(factor.clone(), Expr::integer(*count as i64))
            }
        })
        .collect()
}

fn record_factor(factors: &mut BTreeMap<Expr, usize>, factor: &Expr) {
    if let Some((base, count)) = positive_integer_power(factor) {
        *factors.entry(base).or_insert(0) += count;
    } else {
        *factors.entry(factor.clone()).or_insert(0) += 1;
    }
}

fn retain_common_factors(common: &mut BTreeMap<Expr, usize>, next: &BTreeMap<Expr, usize>) {
    common.retain(|factor, count| {
        let Some(next_count) = next.get(factor).copied() else {
            return false;
        };

        *count = (*count).min(next_count);
        *count > 0
    });
}

fn to_positive_rational(number: &Number) -> BigRational {
    match number {
        Number::Integer(value) => BigRational::from_integer(value.abs()),
        Number::Rational(value) => BigRational::new(value.numer().abs(), value.denom().clone()),
    }
}

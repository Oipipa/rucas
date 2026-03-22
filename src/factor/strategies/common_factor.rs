use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::Signed;

use crate::{
    Expr, ExprKind, Number,
    context::EngineContext,
    factor::{FactorizationResult, FactorizationStatus, FactorizationStrategy, Factorizer},
};

pub(super) fn install(factorizer: &mut Factorizer) {
    factorizer.push_strategy(CommonFactorStrategy);
}

struct CommonFactorStrategy;

impl FactorizationStrategy for CommonFactorStrategy {
    fn name(&self) -> &'static str {
        "common-factor"
    }

    fn try_factor(&self, expr: &Expr, _ctx: &EngineContext) -> Option<FactorizationResult> {
        let ExprKind::Add(terms) = expr.kind() else {
            return None;
        };

        if terms.len() < 2 {
            return None;
        }

        let decomposed_terms: Vec<_> = terms.iter().map(DecomposedTerm::from_expr).collect();
        let common_factor = CommonFactor::from_terms(&decomposed_terms)?;

        let residual = Expr::sum(
            decomposed_terms
                .iter()
                .map(|term| term.divide_by(&common_factor)),
        );
        let factored = Expr::product([common_factor.to_expr(), residual]);

        (factored != *expr).then(|| FactorizationResult {
            expr: factored,
            status: FactorizationStatus::Factored,
            steps: vec!["common factor extracted".to_string()],
        })
    }
}

#[derive(Clone, Debug)]
struct DecomposedTerm {
    coefficient: Number,
    factors: BTreeMap<Expr, usize>,
}

impl DecomposedTerm {
    fn from_expr(expr: &Expr) -> Self {
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

    fn divide_by(&self, common: &CommonFactor) -> Expr {
        let remainder_coefficient = self.coefficient.div(&common.coefficient);
        let mut factors = self.factors.clone();

        for (factor, count) in &common.factors {
            let Some(current) = factors.get_mut(factor) else {
                continue;
            };

            *current -= count;
            if *current == 0 {
                factors.remove(factor);
            }
        }

        let mut rebuilt = Vec::new();
        if !remainder_coefficient.is_one() {
            rebuilt.push(Expr::number(remainder_coefficient));
        }
        rebuilt.extend(rebuild_factors(&factors));
        Expr::product(rebuilt)
    }
}

#[derive(Clone, Debug)]
struct CommonFactor {
    coefficient: Number,
    factors: BTreeMap<Expr, usize>,
}

impl CommonFactor {
    fn from_terms(terms: &[DecomposedTerm]) -> Option<Self> {
        let coefficient = common_numeric_content(terms.iter().map(|term| &term.coefficient))?;
        let factors = common_symbolic_factors(terms);

        if coefficient.is_one() && factors.is_empty() {
            None
        } else {
            Some(Self {
                coefficient,
                factors,
            })
        }
    }

    fn to_expr(&self) -> Expr {
        let mut rebuilt = Vec::new();
        if !self.coefficient.is_one() {
            rebuilt.push(Expr::number(self.coefficient.clone()));
        }
        rebuilt.extend(rebuild_factors(&self.factors));
        Expr::product(rebuilt)
    }
}

fn common_numeric_content<'a>(
    coefficients: impl IntoIterator<Item = &'a Number>,
) -> Option<Number> {
    let mut numer_gcd: Option<BigInt> = None;
    let mut denom_lcm: Option<BigInt> = None;
    let mut all_negative = true;
    let mut saw_any = false;

    for coefficient in coefficients {
        saw_any = true;
        all_negative &= coefficient.is_negative();

        let rational = to_positive_rational(coefficient);
        numer_gcd = Some(match numer_gcd {
            Some(current) => current.gcd(rational.numer()),
            None => rational.numer().clone(),
        });
        denom_lcm = Some(match denom_lcm {
            Some(current) => current.lcm(rational.denom()),
            None => rational.denom().clone(),
        });
    }

    if !saw_any {
        return None;
    }

    let mut content = Number::from_big_rational(BigRational::new(
        numer_gcd.expect("at least one coefficient"),
        denom_lcm.expect("at least one coefficient"),
    ));

    if all_negative {
        content = Number::from_big_rational(-to_positive_rational(&content));
    }

    Some(content)
}

fn common_symbolic_factors(terms: &[DecomposedTerm]) -> BTreeMap<Expr, usize> {
    let Some(first) = terms.first() else {
        return BTreeMap::new();
    };

    let mut common = first.factors.clone();
    common.retain(|factor, count| {
        let mut min_count = *count;

        for term in terms.iter().skip(1) {
            let Some(term_count) = term.factors.get(factor).copied() else {
                return false;
            };
            min_count = min_count.min(term_count);
        }

        *count = min_count;
        *count > 0
    });
    common
}

fn rebuild_factors(factors: &BTreeMap<Expr, usize>) -> Vec<Expr> {
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

fn positive_integer_power(factor: &Expr) -> Option<(Expr, usize)> {
    let ExprKind::Pow { base, exp } = factor.kind() else {
        return None;
    };

    let value = exp.as_number()?.as_i64()?;
    if value <= 1 {
        return None;
    }

    Some((base.clone(), value as usize))
}

fn to_positive_rational(number: &Number) -> BigRational {
    match number {
        Number::Integer(value) => BigRational::from_integer(value.abs()),
        Number::Rational(value) => BigRational::new(value.numer().abs(), value.denom().clone()),
    }
}

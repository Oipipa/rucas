use std::collections::BTreeSet;

use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};

use crate::{
    Expr, Number, Symbol,
    context::EngineContext,
    factor::{FactorizationResult, FactorizationStatus, FactorizationStrategy, Factorizer},
    polynomial::{UnivariatePolynomial, monomial_expr},
};

pub(super) fn install(factorizer: &mut Factorizer) {
    factorizer.push_strategy(PolynomialFactorStrategy);
}

struct PolynomialFactorStrategy;

impl FactorizationStrategy for PolynomialFactorStrategy {
    fn name(&self) -> &'static str {
        "polynomial"
    }

    fn try_factor(&self, expr: &Expr, _ctx: &EngineContext) -> Option<FactorizationResult> {
        let polynomial = UnivariatePolynomial::from_expr(expr)?;
        let (factored, step) = factor_supported_polynomial(&polynomial)?;

        (factored != *expr).then(|| FactorizationResult {
            expr: factored,
            status: FactorizationStatus::Factored,
            steps: vec![step.to_string()],
        })
    }
}

fn factor_supported_polynomial(poly: &UnivariatePolynomial) -> Option<(Expr, &'static str)> {
    let mut assembly = FactorAssembly::with_scale(poly.leading_coefficient()?.clone());
    let mut kind = PolynomialFactorKind::None;

    for (factor, multiplicity) in poly.monic()?.square_free_decomposition()? {
        if multiplicity > 1 {
            kind = kind.max(PolynomialFactorKind::SquareFree);
        }

        kind = kind.max(collect_square_free_factor(
            &factor,
            multiplicity,
            &mut assembly,
        ));
    }

    let factored = assembly.finish();
    (kind != PolynomialFactorKind::None).then_some((factored, kind.step()))
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
enum PolynomialFactorKind {
    #[default]
    None,
    SquareFree,
    DifferenceOfSquares,
    RationalRoots,
}

impl PolynomialFactorKind {
    fn step(self) -> &'static str {
        match self {
            Self::None => "unchanged",
            Self::SquareFree => "square-free factorization",
            Self::DifferenceOfSquares => "difference of squares",
            Self::RationalRoots => "rational root factorization",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FactorAssembly {
    scale: Number,
    factors: Vec<Expr>,
}

impl FactorAssembly {
    fn with_scale(scale: Number) -> Self {
        Self {
            scale,
            factors: Vec::new(),
        }
    }

    fn multiply_scale(&mut self, scalar: &Number) {
        self.scale = self.scale.mul(scalar);
    }

    fn push_factor(&mut self, factor: Expr, multiplicity: usize) {
        if multiplicity == 0 {
            return;
        }

        if multiplicity == 1 {
            self.factors.push(factor);
        } else {
            self.factors
                .push(Expr::pow(factor, Expr::integer(multiplicity as i64)));
        }
    }

    fn finish(mut self) -> Expr {
        if !self.scale.is_one() {
            self.factors.insert(0, Expr::number(self.scale));
        }

        Expr::product(self.factors)
    }
}

fn collect_square_free_factor(
    factor: &UnivariatePolynomial,
    multiplicity: usize,
    assembly: &mut FactorAssembly,
) -> PolynomialFactorKind {
    let mut kind = PolynomialFactorKind::None;
    let mut remaining = factor.clone();

    while let Some(root) = find_rational_root(&remaining) {
        let Some((root_multiplicity, quotient)) = remaining.deflate_linear_root(&root) else {
            break;
        };
        let total_multiplicity = multiplicity * root_multiplicity;

        let (linear_factor, factor_scale) = linear_factor_from_root(remaining.variable(), &root);
        let Some(factor_scale) = factor_scale.powi(total_multiplicity as i64) else {
            break;
        };

        assembly.multiply_scale(&factor_scale);
        assembly.push_factor(linear_factor, total_multiplicity);

        remaining = quotient;
        kind = PolynomialFactorKind::RationalRoots;
    }

    match remaining.degree() {
        None => {}
        Some(0) => assembly.multiply_scale(&remaining.coefficient(0)),
        Some(_) => {
            if let Some((left, right)) = split_difference_of_squares(&remaining) {
                kind = kind.max(PolynomialFactorKind::DifferenceOfSquares);
                kind = kind.max(collect_square_free_factor(&left, multiplicity, assembly));
                kind = kind.max(collect_square_free_factor(&right, multiplicity, assembly));
            } else {
                assembly.push_factor(remaining.to_expr(), multiplicity);
            }
        }
    }

    kind
}

fn find_rational_root(poly: &UnivariatePolynomial) -> Option<Number> {
    if poly.degree()? == 0 {
        return None;
    }

    if poly.evaluate(&Number::zero()).is_zero() {
        return Some(Number::zero());
    }

    rational_root_candidates(poly)?
        .into_iter()
        .find(|candidate| poly.evaluate(candidate).is_zero())
}

fn rational_root_candidates(poly: &UnivariatePolynomial) -> Option<Vec<Number>> {
    let primitive = poly.primitive_part()?;
    let integer_coefficients = primitive.integer_coefficients()?;
    let degree = primitive.degree()?;
    let leading = integer_coefficients.get(&degree)?.abs();
    let constant = integer_coefficients
        .get(&0)
        .cloned()
        .unwrap_or_else(BigInt::zero)
        .abs();

    if leading.is_zero() || constant.is_zero() {
        return Some(Vec::new());
    }

    let numerators = positive_divisors(&constant);
    let denominators = positive_divisors(&leading);
    let mut candidates = BTreeSet::new();

    for numerator in numerators {
        for denominator in &denominators {
            let candidate = Number::rational(numerator.clone(), denominator.clone());
            candidates.insert(candidate.clone());
            candidates.insert(candidate.neg());
        }
    }

    Some(candidates.into_iter().collect())
}

fn positive_divisors(value: &BigInt) -> Vec<BigInt> {
    let value = value.abs();
    if value.is_zero() {
        return Vec::new();
    }

    let mut divisors = BTreeSet::new();
    let limit = value.sqrt();
    let mut candidate = BigInt::one();

    while candidate <= limit {
        if (&value % &candidate).is_zero() {
            divisors.insert(candidate.clone());
            divisors.insert(&value / &candidate);
        }
        candidate += 1;
    }

    divisors.into_iter().collect()
}

fn split_difference_of_squares(
    poly: &UnivariatePolynomial,
) -> Option<(UnivariatePolynomial, UnivariatePolynomial)> {
    let degree = poly.degree()?;
    if degree < 2 || degree % 2 != 0 || poly.coefficients().len() != 2 {
        return None;
    }

    if poly
        .coefficients()
        .keys()
        .any(|power| *power != 0 && *power != degree)
    {
        return None;
    }

    let leading = poly.leading_coefficient()?.clone();
    let constant = poly.coefficient(0);
    let square_leading = leading.sqrt_exact()?;
    let square_constant = constant.neg().sqrt_exact()?;
    let midpoint_degree = degree / 2;

    Some((
        UnivariatePolynomial::from_coefficients(
            poly.variable().clone(),
            [
                (midpoint_degree, square_leading.clone()),
                (0, square_constant.neg()),
            ],
        ),
        UnivariatePolynomial::from_coefficients(
            poly.variable().clone(),
            [(midpoint_degree, square_leading), (0, square_constant)],
        ),
    ))
}

fn linear_factor_from_root(variable: &Symbol, root: &Number) -> (Expr, Number) {
    let (numerator, denominator) = match root {
        Number::Integer(value) => (value.clone(), BigInt::one()),
        Number::Rational(value) => (value.numer().clone(), value.denom().clone()),
    };

    let variable_term = monomial_expr(variable, 1, &Number::integer(denominator.clone()));
    let constant_term = Expr::number(Number::integer(-numerator));
    let factor = Expr::sum([variable_term, constant_term]);
    let scale = Number::rational(1, denominator);

    (factor, scale)
}

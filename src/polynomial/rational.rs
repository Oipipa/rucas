use crate::{Expr, ExprKind, Number, Symbol};

use super::{
    CollectedPolynomial, PolynomialAnalysisLimits, PolynomialCoefficient,
    SparseUnivariatePolynomial, UnivariatePolynomial,
    analysis::{infer_variable, symbols},
    score::{ExpressionScore, expression_score},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnivariateRationalFunction {
    variable: Symbol,
    numerator: UnivariatePolynomial,
    denominator: UnivariatePolynomial,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CollectedRationalFunction {
    variable: Symbol,
    scale: Expr,
    numerator: CollectedPolynomial,
    denominator: CollectedPolynomial,
}

pub(crate) fn collect_rational_expression(expr: &Expr) -> Option<Expr> {
    let limits = PolynomialAnalysisLimits::default();
    let mut best: Option<_> = None;

    if let Some(candidate) = UnivariateRationalFunction::from_expr_with_limits(expr, limits)
        .map(|rational| rational.to_expr())
    {
        update_best_candidate(&mut best, candidate);
    }

    for variable in symbols(expr) {
        let Some(candidate) =
            CollectedRationalFunction::from_expr_with_variable_and_limits(expr, &variable, limits)
                .map(|rational| rational.to_expr())
        else {
            continue;
        };

        update_best_candidate(&mut best, candidate);
    }

    best.and_then(|(_, candidate)| (candidate != *expr).then_some(candidate))
}

impl UnivariateRationalFunction {
    #[cfg(test)]
    pub(crate) fn from_expr(expr: &Expr) -> Option<Self> {
        Self::from_expr_with_limits(expr, PolynomialAnalysisLimits::default())
    }

    pub(crate) fn from_expr_with_limits(
        expr: &Expr,
        limits: PolynomialAnalysisLimits,
    ) -> Option<Self> {
        let variable = infer_variable(expr)?;
        Self::from_expr_with_variable_and_limits(expr, &variable, limits)
    }

    pub(crate) fn from_expr_with_variable_and_limits(
        expr: &Expr,
        variable: &Symbol,
        limits: PolynomialAnalysisLimits,
    ) -> Option<Self> {
        analyze_numeric(expr, variable, limits)
    }

    #[cfg(test)]
    pub(crate) fn numerator(&self) -> &UnivariatePolynomial {
        &self.numerator
    }

    #[cfg(test)]
    pub(crate) fn denominator(&self) -> &UnivariatePolynomial {
        &self.denominator
    }

    pub(crate) fn to_expr(&self) -> Expr {
        if self.denominator.is_one() {
            self.numerator.to_expr()
        } else {
            Expr::product([
                self.numerator.to_expr(),
                Expr::pow(self.denominator.to_expr(), Expr::integer(-1)),
            ])
        }
    }

    fn from_parts(
        variable: Symbol,
        numerator: UnivariatePolynomial,
        denominator: UnivariatePolynomial,
    ) -> Option<Self> {
        Self {
            variable,
            numerator,
            denominator,
        }
        .normalize()
    }

    fn polynomial(variable: &Symbol, polynomial: UnivariatePolynomial) -> Option<Self> {
        Self::from_parts(
            variable.clone(),
            polynomial,
            constant_polynomial(variable, Number::one()),
        )
    }

    fn add(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        let left = self.numerator.mul(&other.denominator, limits)?;
        let right = other.numerator.mul(&self.denominator, limits)?;
        let numerator = left.add(&right, limits)?;
        let denominator = self.denominator.mul(&other.denominator, limits)?;

        Self::from_parts(self.variable.clone(), numerator, denominator)
    }

    fn mul(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        let numerator = self.numerator.mul(&other.numerator, limits)?;
        let denominator = self.denominator.mul(&other.denominator, limits)?;

        Self::from_parts(self.variable.clone(), numerator, denominator)
    }

    fn powi(&self, exponent: i64, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if exponent == 0 {
            return Self::from_parts(
                self.variable.clone(),
                constant_polynomial(&self.variable, Number::one()),
                constant_polynomial(&self.variable, Number::one()),
            );
        }

        let magnitude = exponent.unsigned_abs();
        if magnitude > usize::MAX as u64 {
            return None;
        }

        let magnitude = magnitude as usize;
        let numerator = self.numerator.pow(magnitude, limits)?;
        let denominator = self.denominator.pow(magnitude, limits)?;

        if exponent > 0 {
            Self::from_parts(self.variable.clone(), numerator, denominator)
        } else {
            Self::from_parts(self.variable.clone(), denominator, numerator)
        }
    }

    fn normalize(mut self) -> Option<Self> {
        if self.denominator.is_zero() {
            return None;
        }

        if self.numerator.is_zero() {
            self.denominator = constant_polynomial(&self.variable, Number::one());
            return Some(self);
        }

        let gcd = self.numerator.gcd(&self.denominator)?;
        self.numerator = self.numerator.exact_div(&gcd)?;
        self.denominator = self.denominator.exact_div(&gcd)?;

        let denominator_leading = self.denominator.leading_coefficient()?.clone();
        self.numerator = self.numerator.div_scalar(&denominator_leading)?;
        self.denominator = self.denominator.div_scalar(&denominator_leading)?;

        if self.numerator.is_zero() {
            self.denominator = constant_polynomial(&self.variable, Number::one());
        }

        Some(self)
    }
}

impl CollectedRationalFunction {
    pub(crate) fn from_expr_with_variable_and_limits(
        expr: &Expr,
        variable: &Symbol,
        limits: PolynomialAnalysisLimits,
    ) -> Option<Self> {
        analyze_collected(expr, variable, limits)
    }

    #[cfg(test)]
    pub(crate) fn scale(&self) -> &Expr {
        &self.scale
    }

    #[cfg(test)]
    pub(crate) fn numerator(&self) -> &CollectedPolynomial {
        &self.numerator
    }

    #[cfg(test)]
    pub(crate) fn denominator(&self) -> &CollectedPolynomial {
        &self.denominator
    }

    pub(crate) fn to_expr(&self) -> Expr {
        let rational = if self.denominator.is_one() {
            self.numerator.to_expr()
        } else {
            Expr::product([
                self.numerator.to_expr(),
                Expr::pow(self.denominator.to_expr(), Expr::integer(-1)),
            ])
        };

        if self.scale.is_one() {
            rational
        } else {
            Expr::product([self.scale.clone(), rational])
        }
    }

    fn from_parts(
        variable: Symbol,
        scale: Expr,
        numerator: CollectedPolynomial,
        denominator: CollectedPolynomial,
    ) -> Option<Self> {
        Self {
            variable,
            scale,
            numerator,
            denominator,
        }
        .normalize()
    }

    fn polynomial(variable: &Symbol, polynomial: CollectedPolynomial) -> Option<Self> {
        Self::from_parts(
            variable.clone(),
            Expr::one(),
            polynomial,
            constant_polynomial(variable, Expr::one()),
        )
    }

    fn add(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        if self.denominator == other.denominator {
            let numerator = self
                .numerator
                .mul_expr_factor(&self.scale)
                .add(&other.numerator.mul_expr_factor(&other.scale), limits)?;

            return Self::from_parts(
                self.variable.clone(),
                Expr::one(),
                numerator,
                self.denominator.clone(),
            );
        }

        let left = self
            .numerator
            .mul(&other.denominator, limits)?
            .mul_expr_factor(&self.scale);
        let right = other
            .numerator
            .mul(&self.denominator, limits)?
            .mul_expr_factor(&other.scale);
        let numerator = left.add(&right, limits)?;
        let denominator = self.denominator.mul(&other.denominator, limits)?;

        Self::from_parts(self.variable.clone(), Expr::one(), numerator, denominator)
    }

    fn mul(&self, other: &Self, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if self.variable != other.variable {
            return None;
        }

        let scale = Expr::product([self.scale.clone(), other.scale.clone()]);
        let numerator = self.numerator.mul(&other.numerator, limits)?;
        let denominator = self.denominator.mul(&other.denominator, limits)?;

        Self::from_parts(self.variable.clone(), scale, numerator, denominator)
    }

    fn powi(&self, exponent: i64, limits: PolynomialAnalysisLimits) -> Option<Self> {
        if exponent == 0 {
            return Self::from_parts(
                self.variable.clone(),
                Expr::one(),
                constant_polynomial(&self.variable, Expr::one()),
                constant_polynomial(&self.variable, Expr::one()),
            );
        }

        let magnitude = exponent.unsigned_abs();
        if magnitude > usize::MAX as u64 {
            return None;
        }

        let magnitude = magnitude as usize;
        let scale = Expr::pow(self.scale.clone(), Expr::integer(exponent));
        let numerator = self.numerator.pow(magnitude, limits)?;
        let denominator = self.denominator.pow(magnitude, limits)?;

        if exponent > 0 {
            Self::from_parts(self.variable.clone(), scale, numerator, denominator)
        } else {
            Self::from_parts(self.variable.clone(), scale, denominator, numerator)
        }
    }

    fn normalize(mut self) -> Option<Self> {
        if self.denominator.is_zero() {
            return None;
        }

        if self.scale.is_zero() || self.numerator.is_zero() {
            self.scale = Expr::one();
            self.numerator = constant_polynomial(&self.variable, Expr::zero());
            self.denominator = constant_polynomial(&self.variable, Expr::one());
            return Some(self);
        }

        self.extract_coefficient_content()?;
        self.cancel_numeric_polynomial_gcd()?;

        if self.numerator == self.denominator {
            self.numerator = constant_polynomial(&self.variable, Expr::one());
            self.denominator = constant_polynomial(&self.variable, Expr::one());
        }

        self.pull_constant_polynomial_into_scale(true);
        self.pull_constant_polynomial_into_scale(false);

        if self.scale.is_zero() || self.numerator.is_zero() {
            self.scale = Expr::one();
            self.numerator = constant_polynomial(&self.variable, Expr::zero());
            self.denominator = constant_polynomial(&self.variable, Expr::one());
        }

        Some(self)
    }

    fn extract_coefficient_content(&mut self) -> Option<()> {
        let numerator_content = self.numerator.content()?;
        let denominator_content = self.denominator.content()?;

        self.numerator = self.numerator.primitive_part()?;
        self.denominator = self.denominator.primitive_part()?;
        self.scale = Expr::product([
            self.scale.clone(),
            numerator_content,
            Expr::pow(denominator_content, Expr::integer(-1)),
        ]);

        Some(())
    }

    fn cancel_numeric_polynomial_gcd(&mut self) -> Option<()> {
        let Some(numerator) = self.numerator.numeric_coefficients() else {
            return Some(());
        };
        let Some(denominator) = self.denominator.numeric_coefficients() else {
            return Some(());
        };

        let gcd = numerator.gcd(&denominator)?;
        let mut numerator = numerator.exact_div(&gcd)?;
        let mut denominator = denominator.exact_div(&gcd)?;
        let denominator_leading = denominator.leading_coefficient()?.clone();

        numerator = numerator.div_scalar(&denominator_leading)?;
        denominator = denominator.div_scalar(&denominator_leading)?;

        self.numerator = CollectedPolynomial::from_numeric_coefficients(&numerator);
        self.denominator = CollectedPolynomial::from_numeric_coefficients(&denominator);

        Some(())
    }

    fn pull_constant_polynomial_into_scale(&mut self, numerator: bool) {
        let coefficient = if numerator {
            if self.numerator.degree() != Some(0) {
                return;
            }
            self.numerator.coefficient(0)
        } else {
            if self.denominator.degree() != Some(0) {
                return;
            }
            self.denominator.coefficient(0)
        };

        if coefficient.is_zero() {
            return;
        }

        if numerator {
            self.scale = Expr::product([self.scale.clone(), coefficient.clone()]);
            self.numerator = constant_polynomial(&self.variable, Expr::one());
        } else {
            self.scale = Expr::product([
                self.scale.clone(),
                Expr::pow(coefficient.clone(), Expr::integer(-1)),
            ]);
            self.denominator = constant_polynomial(&self.variable, Expr::one());
        }
    }
}

fn analyze_numeric(
    expr: &Expr,
    variable: &Symbol,
    limits: PolynomialAnalysisLimits,
) -> Option<UnivariateRationalFunction> {
    match expr.kind() {
        ExprKind::Add(terms) => {
            let mut rational = UnivariateRationalFunction::polynomial(
                variable,
                constant_polynomial(variable, Number::zero()),
            )?;

            for term in terms {
                rational = rational.add(&analyze_numeric(term, variable, limits)?, limits)?;
            }

            Some(rational)
        }
        ExprKind::Mul(factors) => {
            let mut rational = UnivariateRationalFunction::polynomial(
                variable,
                constant_polynomial(variable, Number::one()),
            )?;

            for factor in factors {
                rational = rational.mul(&analyze_numeric(factor, variable, limits)?, limits)?;
            }

            Some(rational)
        }
        ExprKind::Pow { base, exp } => {
            let exponent = exp.as_number()?.as_i64()?;
            analyze_numeric(base, variable, limits)?.powi(exponent, limits)
        }
        ExprKind::Number(_) | ExprKind::Symbol(_) => UnivariateRationalFunction::polynomial(
            variable,
            UnivariatePolynomial::from_expr_with_variable_and_limits(expr, variable, limits)?,
        ),
        ExprKind::Call { .. } | ExprKind::Derivative(_) | ExprKind::Integral(_) => None,
    }
}

fn analyze_collected(
    expr: &Expr,
    variable: &Symbol,
    limits: PolynomialAnalysisLimits,
) -> Option<CollectedRationalFunction> {
    match expr.kind() {
        ExprKind::Add(terms) => {
            let mut rational = CollectedRationalFunction::polynomial(
                variable,
                constant_polynomial(variable, Expr::zero()),
            )?;

            for term in terms {
                rational = rational.add(&analyze_collected(term, variable, limits)?, limits)?;
            }

            Some(rational)
        }
        ExprKind::Mul(factors) => {
            let mut rational = CollectedRationalFunction::polynomial(
                variable,
                constant_polynomial(variable, Expr::one()),
            )?;

            for factor in factors {
                rational = rational.mul(&analyze_collected(factor, variable, limits)?, limits)?;
            }

            Some(rational)
        }
        ExprKind::Pow { base, exp } => {
            let exponent = exp.as_number()?.as_i64()?;
            analyze_collected(base, variable, limits)?.powi(exponent, limits)
        }
        ExprKind::Number(_)
        | ExprKind::Symbol(_)
        | ExprKind::Call { .. }
        | ExprKind::Derivative(_)
        | ExprKind::Integral(_) => CollectedRationalFunction::polynomial(
            variable,
            CollectedPolynomial::from_expr_with_variable_and_limits(expr, variable, limits)?,
        ),
    }
}

fn update_best_candidate(best: &mut Option<(ExpressionScore, Expr)>, candidate: Expr) {
    let candidate_score = expression_score(&candidate);
    match best {
        Some((current_score, _)) if candidate_score >= *current_score => {}
        _ => *best = Some((candidate_score, candidate)),
    }
}

fn constant_polynomial<C: PolynomialCoefficient>(
    variable: &Symbol,
    value: C,
) -> SparseUnivariatePolynomial<C> {
    SparseUnivariatePolynomial::from_coefficients(variable.clone(), [(0, value)])
}

use std::{fmt, sync::Arc};

use super::{Function, Number, Symbol, canon};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Expr(Arc<ExprNode>);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct ExprNode {
    kind: ExprKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DerivativeExpr {
    pub expr: Expr,
    pub variable: Symbol,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct IntegralExpr {
    pub expr: Expr,
    pub variable: Symbol,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ExprKind {
    Number(Number),
    Symbol(Symbol),
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    Pow { base: Expr, exp: Expr },
    Call { function: Function, args: Vec<Expr> },
    Derivative(DerivativeExpr),
    Integral(IntegralExpr),
}

impl Expr {
    pub(crate) fn raw(kind: ExprKind) -> Self {
        Self(Arc::new(ExprNode { kind }))
    }

    pub fn kind(&self) -> &ExprKind {
        &self.0.kind
    }

    pub fn number(number: Number) -> Self {
        Self::raw(ExprKind::Number(number))
    }

    pub fn integer(value: i64) -> Self {
        Self::number(Number::integer(value))
    }

    pub fn rational(numer: i64, denom: i64) -> Self {
        Self::number(Number::rational(numer, denom))
    }

    pub fn zero() -> Self {
        Self::integer(0)
    }

    pub fn one() -> Self {
        Self::integer(1)
    }

    pub fn from_symbol(symbol: Symbol) -> Self {
        Self::raw(ExprKind::Symbol(symbol))
    }

    pub fn symbol(name: impl Into<String>) -> Self {
        Self::from_symbol(Symbol::new(name))
    }

    pub fn sum(terms: impl IntoIterator<Item = Expr>) -> Self {
        canon::sum(terms)
    }

    pub fn product(factors: impl IntoIterator<Item = Expr>) -> Self {
        canon::product(factors)
    }

    pub fn pow(base: Expr, exp: Expr) -> Self {
        canon::pow(base, exp)
    }

    pub fn call(function: Function, args: impl IntoIterator<Item = Expr>) -> Self {
        Self::raw(ExprKind::Call {
            function,
            args: args.into_iter().collect(),
        })
    }

    pub fn derivative(expr: Expr, variable: Symbol) -> Self {
        Self::raw(ExprKind::Derivative(DerivativeExpr { expr, variable }))
    }

    pub fn integral(expr: Expr, variable: Symbol) -> Self {
        Self::raw(ExprKind::Integral(IntegralExpr { expr, variable }))
    }

    pub fn as_number(&self) -> Option<&Number> {
        match self.kind() {
            ExprKind::Number(number) => Some(number),
            _ => None,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.as_number().is_some_and(Number::is_zero)
    }

    pub fn is_one(&self) -> bool {
        self.as_number().is_some_and(Number::is_one)
    }

    fn precedence(&self) -> u8 {
        match self.kind() {
            ExprKind::Add(_) => 1,
            ExprKind::Mul(_) => 2,
            ExprKind::Pow { .. } => 3,
            ExprKind::Number(_)
            | ExprKind::Symbol(_)
            | ExprKind::Call { .. }
            | ExprKind::Derivative(_)
            | ExprKind::Integral(_) => 4,
        }
    }

    fn needs_pow_operand_parentheses(&self) -> bool {
        match self.kind() {
            ExprKind::Number(number) => number.is_negative() || !number.is_integer(),
            ExprKind::Symbol(_) | ExprKind::Call { .. } => false,
            ExprKind::Add(_)
            | ExprKind::Mul(_)
            | ExprKind::Pow { .. }
            | ExprKind::Derivative(_)
            | ExprKind::Integral(_) => true,
        }
    }

    fn fmt_with_precedence(&self, f: &mut fmt::Formatter<'_>, parent: u8) -> fmt::Result {
        let needs_parens = self.precedence() < parent;
        if needs_parens {
            write!(f, "(")?;
        }

        match self.kind() {
            ExprKind::Number(number) => write!(f, "{number}")?,
            ExprKind::Symbol(symbol) => write!(f, "{symbol}")?,
            ExprKind::Add(terms) => {
                for (index, term) in terms.iter().enumerate() {
                    if index > 0 {
                        write!(f, " + ")?;
                    }
                    term.fmt_with_precedence(f, 1)?;
                }
            }
            ExprKind::Mul(factors) => {
                for (index, factor) in factors.iter().enumerate() {
                    if index > 0 {
                        write!(f, " * ")?;
                    }
                    factor.fmt_with_precedence(f, 2)?;
                }
            }
            ExprKind::Pow { base, exp } => {
                if base.needs_pow_operand_parentheses() {
                    write!(f, "(")?;
                    base.fmt_with_precedence(f, 0)?;
                    write!(f, ")")?;
                } else {
                    base.fmt_with_precedence(f, 3)?;
                }
                write!(f, "^")?;
                if exp.needs_pow_operand_parentheses() {
                    write!(f, "(")?;
                    exp.fmt_with_precedence(f, 0)?;
                    write!(f, ")")?;
                } else {
                    exp.fmt_with_precedence(f, 3)?;
                }
            }
            ExprKind::Call { function, args } => {
                write!(f, "{function}(")?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    arg.fmt_with_precedence(f, 0)?;
                }
                write!(f, ")")?;
            }
            ExprKind::Derivative(derivative) => {
                write!(f, "d/d{}(", derivative.variable)?;
                derivative.expr.fmt_with_precedence(f, 0)?;
                write!(f, ")")?;
            }
            ExprKind::Integral(integral) => {
                write!(f, "int(")?;
                integral.expr.fmt_with_precedence(f, 0)?;
                write!(f, ") d{}", integral.variable)?;
            }
        }

        if needs_parens {
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_precedence(f, 0)
    }
}

impl From<i64> for Expr {
    fn from(value: i64) -> Self {
        Self::integer(value)
    }
}

impl From<Number> for Expr {
    fn from(number: Number) -> Self {
        Self::number(number)
    }
}

impl From<Symbol> for Expr {
    fn from(symbol: Symbol) -> Self {
        Self::from_symbol(symbol)
    }
}

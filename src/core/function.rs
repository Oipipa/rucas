use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BuiltinFunction {
    Sin,
    Cos,
    Exp,
    Log,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Function {
    Builtin(BuiltinFunction),
    Named(String),
}

impl Function {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Exp => "exp",
            Self::Log => "log",
        };

        write!(f, "{name}")
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin(function) => write!(f, "{function}"),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}

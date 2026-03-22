use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Assumptions {
    pub real: bool,
    pub positive: bool,
    pub integer: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Symbol {
    name: String,
    assumptions: Assumptions,
}

impl Symbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            assumptions: Assumptions::default(),
        }
    }

    pub fn with_assumptions(name: impl Into<String>, assumptions: Assumptions) -> Self {
        Self {
            name: name.into(),
            assumptions,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn assumptions(&self) -> &Assumptions {
        &self.assumptions
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

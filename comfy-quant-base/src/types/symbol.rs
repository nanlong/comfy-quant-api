use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Default, sqlx::Type, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Symbol(s.into())
    }
}

impl From<String> for Symbol {
    fn from(value: String) -> Self {
        Symbol::new(value)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Symbol::new(value)
    }
}

impl From<&Symbol> for Symbol {
    fn from(value: &Symbol) -> Self {
        value.clone()
    }
}

impl From<&Symbol> for String {
    fn from(value: &Symbol) -> Self {
        value.to_string()
    }
}

impl From<Symbol> for String {
    fn from(value: Symbol) -> Self {
        value.to_string()
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Exchange(String);

impl Exchange {
    pub fn new(s: impl Into<String>) -> Self {
        Exchange(s.into())
    }
}

impl From<String> for Exchange {
    fn from(value: String) -> Self {
        Exchange::new(value)
    }
}

impl From<&str> for Exchange {
    fn from(value: &str) -> Self {
        Exchange::new(value)
    }
}

impl AsRef<str> for Exchange {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

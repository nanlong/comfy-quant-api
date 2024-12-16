use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Default, sqlx::Type, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Exchange {
    Binance,
    #[default]
    Unknow,
}

impl From<&str> for Exchange {
    fn from(value: &str) -> Self {
        match value {
            "binance" => Exchange::Binance,
            _ => Exchange::Unknow,
        }
    }
}

impl From<String> for Exchange {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&Exchange> for Exchange {
    fn from(value: &Exchange) -> Self {
        value.clone()
    }
}

impl AsRef<str> for Exchange {
    fn as_ref(&self) -> &str {
        match self {
            Exchange::Binance => "binance",
            Exchange::Unknow => "unknow",
        }
    }
}

impl From<Exchange> for String {
    fn from(value: Exchange) -> Self {
        value.as_ref().to_string()
    }
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

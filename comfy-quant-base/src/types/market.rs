use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Default, sqlx::Type, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Market {
    #[default]
    Spot, // 现货
    Usdm,    // U本位合约
    Coinm,   // 币本位合约
    Vanilla, // 期货
}

impl From<&str> for Market {
    fn from(value: &str) -> Self {
        match value {
            "spot" => Market::Spot,
            "usdm" => Market::Usdm,
            "coinm" => Market::Coinm,
            "vanilla" => Market::Vanilla,
            _ => Market::Spot,
        }
    }
}

impl From<String> for Market {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&Market> for Market {
    fn from(value: &Market) -> Self {
        value.clone()
    }
}

impl AsRef<str> for Market {
    fn as_ref(&self) -> &str {
        match self {
            Market::Spot => "spot",
            Market::Usdm => "usdm",
            Market::Coinm => "coinm",
            Market::Vanilla => "vanilla",
        }
    }
}

impl fmt::Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum FuturesMarket {
    Usdm,    // U本位合约
    Coinm,   // 币本位合约
    Vanilla, // 期货
}

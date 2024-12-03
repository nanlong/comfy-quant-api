use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Market {
    Spot,
    Futures,
}

impl From<Market> for String {
    fn from(value: Market) -> Self {
        value.as_ref().to_string()
    }
}

impl FromStr for Market {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "spot" => Self::Spot,
            "futures" => Self::Futures,
            _ => anyhow::bail!("Invalid market"),
        })
    }
}

impl TryFrom<&str> for Market {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for Market {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl AsRef<str> for Market {
    fn as_ref(&self) -> &str {
        match self {
            Self::Spot => "spot",
            Self::Futures => "futures",
        }
    }
}

impl fmt::Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

use super::{Futures, Spot};

pub struct BinanceClient {
    pub(crate) api_key: String,
    pub(crate) secret_key: String,
}

impl BinanceClient {
    pub fn new(api_key: String, secret_key: String) -> Self {
        BinanceClient {
            api_key,
            secret_key,
        }
    }

    pub fn spot(&self) -> Spot {
        Spot::new(self)
    }

    pub fn futures(&self) -> Futures {
        Futures::new(self)
    }
}

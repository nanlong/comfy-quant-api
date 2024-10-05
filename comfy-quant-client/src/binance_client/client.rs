use super::{Futures, Spot};
use bon::bon;

#[derive(Debug, Clone)]
pub struct BinanceClient {
    pub(crate) api_key: Option<String>,
    pub(crate) secret_key: Option<String>,
}

#[bon]
impl BinanceClient {
    #[builder]
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
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

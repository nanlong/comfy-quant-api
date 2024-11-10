use super::{Futures, Spot};
use binance::config::Config;
use bon::bon;

#[derive(Debug, Clone)]
pub struct BinanceClient {
    pub(crate) api_key: Option<String>,
    pub(crate) secret_key: Option<String>,
    pub(crate) config: Option<Config>,
}

#[bon]
impl BinanceClient {
    #[builder(on(String, into))]
    pub fn new(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: Option<Config>,
    ) -> Self {
        BinanceClient {
            api_key,
            secret_key,
            config,
        }
    }

    pub fn spot(&self) -> Spot {
        Spot::new(self)
    }

    pub fn futures(&self) -> Futures {
        Futures::new(self)
    }
}

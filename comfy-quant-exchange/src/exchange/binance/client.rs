use super::{Futures, FuturesWebsocket, Spot, SpotWebsocket};
use binance::{config::Config, futures::websockets::FuturesMarket};
use bon::bon;

#[derive(Debug, Clone)]
pub struct BinanceClient {
    api_key: Option<String>,
    secret_key: Option<String>,
    config: Option<Config>,
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

    pub fn spot_websocket(&self, topic: impl Into<String>) -> SpotWebsocket {
        SpotWebsocket::new(self, topic)
    }

    pub fn futures(&self) -> Futures {
        Futures::new(self)
    }

    pub fn futures_websocket(
        &self,
        market: FuturesMarket,
        topic: impl Into<String>,
    ) -> FuturesWebsocket {
        FuturesWebsocket::new(self, market, topic)
    }

    pub fn config(&self) -> &Option<Config> {
        &self.config
    }

    pub(crate) fn create_api<T, F1, F2>(&self, new: F1, new_with_config: F2) -> T
    where
        F1: FnOnce(Option<String>, Option<String>) -> T,
        F2: FnOnce(Option<String>, Option<String>, &Config) -> T,
    {
        let api_key = self.api_key.clone();
        let secret_key = self.secret_key.clone();

        self.config
            .as_ref()
            .map_or(new(api_key.clone(), secret_key.clone()), |config| {
                new_with_config(api_key, secret_key, config)
            })
    }
}

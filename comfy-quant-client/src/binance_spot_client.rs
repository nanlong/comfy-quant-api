use binance::{account::Account, api::Binance};

#[derive(Debug)]
pub struct BinanceSpotClient {
    api_key: String,
    secret_key: String,
}

impl BinanceSpotClient {
    pub fn new(api_key: String, secret_key: String) -> Self {
        // let client = Account::new(Some(api_key), Some(secret_key));

        BinanceSpotClient {
            api_key,
            secret_key,
        }
    }

    pub fn get_account(&self) -> Account {
        Account::new(Some(self.api_key.clone()), Some(self.secret_key.clone()))
    }
}

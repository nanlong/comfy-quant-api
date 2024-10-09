use anyhow::Result;
use binance::{account::Account, api::Binance, model::AccountInformation};

pub struct BinanceSpotClient {
    account: Account,
}

impl BinanceSpotClient {
    pub fn new(api_key: String, secret_key: String) -> Self {
        let account = Account::new(Some(api_key), Some(secret_key));

        BinanceSpotClient { account }
    }

    pub fn get_account(&self) -> Result<AccountInformation> {
        let account_information = self
            .account
            .get_account()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(account_information)
    }
}

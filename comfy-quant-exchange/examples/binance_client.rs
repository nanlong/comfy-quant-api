// use binance::{api, config::Config};
use comfy_quant_exchange::client::{
    spot_client::binance_spot_client::BinanceSpotClient, spot_client_kind::SpotClientExecutable,
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = env::var("BINANCE_API_KEY2")?;
    let secret_key = env::var("BINANCE_SECRET_KEY2")?;

    dbg!(&api_key);
    dbg!(&secret_key);

    let client = BinanceSpotClient::builder()
        .api_key(api_key)
        .secret_key(secret_key)
        // .config(Config::testnet())
        .build();

    let account = client.get_account().await?;

    dbg!(&account);

    // let symbol = client.get_symbol_info("btc", "usdt").await?;

    // dbg!(symbol);

    Ok(())
}

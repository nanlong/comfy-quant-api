use comfy_quant_exchange::client::{
    spot_client::binance_spot_client::BinanceSpotClient, spot_client_kind::SpotClientExecutable,
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let api_key = env::var("BINANCE_TESTNET_API_KEY")?;
    // let secret_key = env::var("BINANCE_TESTNET_SECRET_KEY")?;

    let api_key = env::var("BINANCE_API_KEY2")?;
    let secret_key = env::var("BINANCE_SECRET_KEY2")?;

    dbg!(&api_key);
    dbg!(&secret_key);

    let client = BinanceSpotClient::builder()
        .api_key(api_key)
        .secret_key(secret_key)
        // .config(binance::config::Config::testnet())
        .build();

    // let account = client.get_account().await?;

    // dbg!(&account);

    let balance = client.get_balance("dot").await?;

    dbg!(&balance);

    let symbol = client.get_symbol_info("dot", "usdt").await?;

    dbg!(symbol);

    // let price = client.get_price("btc", "usdt").await?;

    // dbg!(&price);

    // let order = client.market_buy("btc", "usdt", 0.00015).await?;

    // dbg!(&order);

    Ok(())
}

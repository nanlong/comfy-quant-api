use anyhow::Result;
use comfy_quant_client::BinanceClient;
use std::env;

fn main() -> Result<()> {
    let api_key = env::var("BINANCE_API_KEY2")?;
    let secret_key = env::var("BINANCE_SECRET_KEY2")?;

    println!("api_key: {}", api_key);
    println!("secret_key: {}", secret_key);

    let client = BinanceClient::new(api_key, secret_key);
    let account_information = client.spot().get_account()?;
    println!("{:?}", account_information);

    let account_information = client.futures().get_account()?;
    println!("{:?}", account_information);
    Ok(())
}

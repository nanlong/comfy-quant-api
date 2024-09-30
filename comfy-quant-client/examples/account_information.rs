use anyhow::Result;
use comfy_quant_client::BinanceSpotClient;
use std::env;

fn main() -> Result<()> {
    let api_key = env::var("BINANCE_API_KEY")?;
    let secret_key = env::var("BINANCE_SECRET_KEY")?;

    println!("api_key: {}", api_key);
    println!("secret_key: {}", secret_key);

    let client = BinanceSpotClient::new(api_key, secret_key);
    let account_information = client.get_account()?;
    println!("{:?}", account_information);
    Ok(())
}

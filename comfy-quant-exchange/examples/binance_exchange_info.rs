use comfy_quant_exchange::exchange::binance::BinanceClient;

fn main() -> anyhow::Result<()> {
    let client = BinanceClient::builder().build();

    let info = client.spot().get_exchange_info()?;

    dbg!(info);

    Ok(())
}

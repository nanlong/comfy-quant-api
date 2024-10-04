use anyhow::Result;
use binance::model::KlineSummaries;
use comfy_quant_client::binance_client::BinanceClient;

fn main() -> Result<()> {
    let client = BinanceClient::builder().build();

    let klines = client.spot().get_klines(
        "BTCUSDT",
        "1d",
        10,
        Some(1502928000000),
        Some(1503359999999),
    )?;

    match klines {
        KlineSummaries::AllKlineSummaries(klines) => {
            println!("klines count: {:?}", klines.len());

            for kline in klines {
                println!("{:?}", kline);
            }
        }
        _ => {}
    }

    Ok(())
}

// open_time: 1719446400000,
// open: "60864.98000000",
// high: "62389.22000000",
// low: "60606.63000000",
// close: "61706.47000000",
// volume: "18344.28631000",
// close_time: 1719532799999,
// quote_asset_volume: "1126705164.78289200",
// number_of_trades: 1062176,
// taker_buy_base_asset_volume: "9298.26500000",
// taker_buy_quote_asset_volume: "570914131.05718340"

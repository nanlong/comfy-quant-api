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

    let account = client.get_account().await?;

    dbg!(&account);

    // [comfy-quant-exchange/examples/binance_client.rs:25:5] &account = AccountInformation {
    //     maker_commission_rate: 0.001,
    //     taker_commission_rate: 0.001,
    //     can_trade: true,
    // }

    // let balance = client.get_balance("dot").await?;

    // dbg!(&balance);

    // let symbol = client.get_symbol_info("dot", "usdt").await?;

    // dbg!(symbol);

    // let price = client.get_price("btc", "usdt").await?;

    // dbg!(&price);

    // let order = client.market_buy("dot", "usdt", 1.).await?;

    // dbg!(&order);

    // let balance = client.get_balance("dot").await?;

    // dbg!(&balance);

    Ok(())
}

// &order = Order {
//     symbol: Symbol(
//         "DOTUSDT",
//     ),
//     order_id: "4545425503",
//     client_order_id: Some(
//         "rjpFppW6rW70yLMRilDQJ9",
//     ),
//     price: "0",
//     avg_price: "5.68",
//     orig_qty: "1",
//     executed_qty: "1",
//     cumulative_quote_qty: "5.68",
//     order_type: Market,
//     order_side: Sell,
//     order_status: Filled,
//     time: 0,
//     update_time: 0,
// }

// &balance = Balance {
//     asset: "USDT",
//     free: "5.85521829",
//     locked: "0.00000000",
// }

// &balance = Balance {
//     asset: "USDT",
//     free: "11.53952829",
//     locked: "0.00000000",
// }

// [comfy-quant-exchange/examples/binance_client.rs:41:5] &order = Order {
//     symbol: Symbol(
//         "DOTUSDT",
//     ),
//     order_id: "4545442857",
//     client_order_id: Some(
//         "RJVi3a54eRxghDUnexrdrL",
//     ),
//     price: "0",
//     avg_price: "5.69",
//     orig_qty: "1",
//     executed_qty: "1",
//     cumulative_quote_qty: "5.69",
//     order_type: Market,
//     order_side: Sell,
//     order_status: Filled,
//     time: 0,
//     update_time: 0,
// }

// [comfy-quant-exchange/examples/binance_client.rs:41:5] &order = Order {
//     symbol: Symbol(
//         "DOTUSDT",
//     ),
//     order_id: "4545473317",
//     client_order_id: Some(
//         "ajgPsjQslSMpaE5pRsMD2I",
//     ),
//     price: "0",
//     avg_price: "5.653",
//     orig_qty: "1",
//     executed_qty: "1",
//     cumulative_quote_qty: "5.653",
//     order_type: Market,
//     order_side: Buy,
//     order_status: Filled,
//     time: 0,
//     update_time: 0,
// }
// [comfy-quant-exchange/examples/binance_client.rs:45:5] &balance = Balance {
//     asset: "DOT",
//     free: "1.99900000",
//     locked: "0.00000000",
// }

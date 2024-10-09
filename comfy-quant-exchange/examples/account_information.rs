use anyhow::Result;
use comfy_quant_exchange::BinanceClient;
use std::env;

fn main() -> Result<()> {
    let api_key = env::var("BINANCE_API_KEY2")?;
    let secret_key = env::var("BINANCE_SECRET_KEY2")?;

    println!("api_key: {}", api_key);
    println!("secret_key: {}", secret_key);

    let client = BinanceClient::builder()
        .api_key(api_key)
        .secret_key(secret_key)
        .build();
    // let account_information = client.spot().get_account()?;
    // println!("{:?}", account_information);

    let symbol_info = client.spot().get_symbol_info("BTCUSDT")?;
    println!("{:?}", symbol_info);
    // Symbol { symbol: "DOTUSDT", status: "TRADING", base_asset: "DOT", base_asset_precision: 8, quote_asset: "USDT", quote_precision: 8, order_types: ["LIMIT", "LIMIT_MAKER", "MARKET", "STOP_LOSS", "STOP_LOSS_LIMIT", "TAKE_PROFIT", "TAKE_PROFIT_LIMIT"], iceberg_allowed: true, is_spot_trading_allowed: true, is_margin_trading_allowed: true, filters: [PriceFilter { min_price: "0.00100000", max_price: "10000.00000000", tick_size: "0.00100000" }, LotSize { min_qty: "0.01000000", max_qty: "90000.00000000", step_size: "0.01000000" }, IcebergParts { limit: Some(10) }, MarketLotSize { min_qty: "0.00000000", max_qty: "131774.59812500", step_size: "0.00000000" }, TrailingData { min_trailing_above_delta: Some(10), max_trailing_above_delta: Some(2000), min_trailing_below_delta: Some(10), max_trailing_below_delta: Some(2000) }, PercentPriceBySide { bid_multiplier_up: "5", bid_multiplier_down: "0.2", ask_multiplier_up: "5", ask_multiplier_down: "0.2", avg_price_mins: Some(5.0) }, Notional { notional: None, min_notional: Some("5.00000000"), apply_to_market: None, avg_price_mins: Some(5.0) }, MaxNumOrders { max_num_orders: Some(200) }, MaxNumAlgoOrders { max_num_algo_orders: Some(5) }] }

    // filters LotSize { min_qty: "0.01000000", max_qty: "90000.00000000", step_size: "0.01000000" }
    // filters Notional { notional: None, min_notional: Some("5.00000000"), apply_to_market: None, avg_price_mins: Some(5.0) }

    // let account_information = client.futures().get_account()?;
    // println!("{:?}", account_information);
    Ok(())

    // Symbol { symbol: "BTCUSDT", status: "TRADING", base_asset: "BTC", base_asset_precision: 8, quote_asset: "USDT", quote_precision: 8, order_types: ["LIMIT", "LIMIT_MAKER", "MARKET", "STOP_LOSS", "STOP_LOSS_LIMIT", "TAKE_PROFIT", "TAKE_PROFIT_LIMIT"], iceberg_allowed: true, is_spot_trading_allowed: true, is_margin_trading_allowed: true, filters: [PriceFilter { min_price: "0.01000000", max_price: "1000000.00000000", tick_size: "0.01000000" }, LotSize { min_qty: "0.00001000", max_qty: "9000.00000000", step_size: "0.00001000" }, IcebergParts { limit: Some(10) }, MarketLotSize { min_qty: "0.00000000", max_qty: "104.43017941", step_size: "0.00000000" }, TrailingData { min_trailing_above_delta: Some(10), max_trailing_above_delta: Some(2000), min_trailing_below_delta: Some(10), max_trailing_below_delta: Some(2000) }, PercentPriceBySide { bid_multiplier_up: "5", bid_multiplier_down: "0.2", ask_multiplier_up: "5", ask_multiplier_down: "0.2", avg_price_mins: Some(5.0) }, Notional { notional: None, min_notional: Some("5.00000000"), apply_to_market: None, avg_price_mins: Some(5.0) }, MaxNumOrders { max_num_orders: Some(200) }, MaxNumAlgoOrders { max_num_algo_orders: Some(5) }] }

    // LotSize { min_qty: "0.00001000", max_qty: "9000.00000000", step_size: "0.00001000" }
    // Notional { notional: None, min_notional: Some("5.00000000"), apply_to_market: None, avg_price_mins: Some(5.0) }
}

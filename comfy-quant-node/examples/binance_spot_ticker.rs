use comfy_quant_node::{
    data_source::binance_spot_ticker::{BinanceSpotTicker, BinanceSpotTickerInput},
    traits::node::Node,
};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let node = BinanceSpotTicker {};
    let input = BinanceSpotTickerInput {
        base_currency: "BTC".to_string(),
        quote_currency: "USDT".to_string(),
    };

    let (tx, mut rx) = comfy_quant_node::utils::create_mpsc_channel();

    tokio::spawn(async move {
        node.execute(input, tx).await.unwrap();
    });

    while let Some(ticker) = rx.next().await {
        println!("{:?}", ticker);
    }
}

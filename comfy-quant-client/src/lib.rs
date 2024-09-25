mod behavior;
mod binance_spot_client;

use behavior::ClientBehavior;
use binance_spot_client::BinanceSpotClient;
use enum_dispatch::enum_dispatch;

#[derive(Debug)]
#[enum_dispatch]
pub enum Client {
    BinanceSpotClient(BinanceSpotClient),
}

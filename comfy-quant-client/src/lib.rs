mod behavior;
pub mod binance_client;
mod binance_spot_client;
pub mod subscription;

use behavior::ClientBehavior;
pub use binance_client::BinanceClient;
pub use binance_spot_client::BinanceSpotClient;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub enum Client {
    BinanceSpotClient(BinanceSpotClient),
}

mod client_service;
mod port;
mod slot;
mod slots;
mod stats;
mod symbol_price_store;
mod tick;
mod traits;

pub(crate) use port::Port;
pub(crate) use slot::Slot;
pub(crate) use stats::Stats;
pub(crate) use symbol_price_store::SymbolPriceStore;
pub(crate) use tick::Tick;
pub(crate) use traits::SymbolPriceStorable;

pub use client_service::SpotClientService;
pub use traits::{
    Connectable, Executable, NodeName, NodeStats, NodeSymbolPrice, PortAccessor, Setupable,
    SpotTradeable, StrategyStats,
};

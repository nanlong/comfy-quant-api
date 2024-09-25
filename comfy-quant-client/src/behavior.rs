use crate::Client;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(Client)]
pub trait ClientBehavior {}

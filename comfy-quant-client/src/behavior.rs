use enum_dispatch::enum_dispatch;

#[allow(unused)]
#[enum_dispatch(Client)]
pub trait ClientBehavior {}

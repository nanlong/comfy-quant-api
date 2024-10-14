use std::{
    fmt,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug)]
pub struct Slot<T>(pub T);

impl<T> Deref for Slot<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Slot<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Slot<T>
where
    T: Clone + fmt::Debug + Send + Sync + 'static,
{
    pub fn new(data: T) -> Self {
        Slot(data)
    }
}

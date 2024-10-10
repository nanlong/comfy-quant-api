use bon::bon;
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

#[bon]
impl<T> Slot<T>
where
    T: Clone + fmt::Debug + Send + Sync + 'static,
{
    #[builder]
    pub fn new(data: T) -> Self {
        Slot(data)
    }

    pub fn data(&self) -> &T {
        &self.0
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slot_builder() -> anyhow::Result<()> {
        let slot = Slot::<usize>::builder().data(10).build();
        assert_eq!(slot.data(), &10);
        Ok(())
    }
}

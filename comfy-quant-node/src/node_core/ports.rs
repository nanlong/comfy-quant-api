use super::{slot::Slot, slots::Slots};
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug)]
pub struct Ports {
    inputs: Slots,
    outputs: Slots,
}

impl Ports {
    pub fn new() -> Self {
        Self {
            inputs: Slots::new(),
            outputs: Slots::new(),
        }
    }

    pub fn add_input<T>(&mut self, index: usize, slot: Arc<Slot<T>>) -> Result<()>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.inputs.set(index, slot);

        Ok(())
    }

    pub fn get_input<T>(&self, index: usize) -> Result<Arc<Slot<T>>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let slot = self
            .inputs
            .get::<Arc<Slot<T>>>(index)
            .and_then(|s| Some(Arc::clone(s)))
            .ok_or_else(|| anyhow::anyhow!("Input slot {} is not connected", index))?;

        Ok(slot)
    }

    pub fn add_output<T>(&mut self, index: usize, slot: Slot<T>) -> Result<()>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.outputs.set(index, Arc::new(slot));

        Ok(())
    }

    pub fn get_output<T>(&self, index: usize) -> Result<Arc<Slot<T>>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let slot = self
            .outputs
            .get::<Arc<Slot<T>>>(index)
            .and_then(|s| Some(Arc::clone(s)))
            .ok_or_else(|| anyhow::anyhow!("Output slot {} is not connected", index))?;

        Ok(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ports_add_output() {
        let mut ports = Ports::new();

        // Add input
        let slot = Arc::new(Slot::<usize>::builder().data(5).build());
        ports.add_input(0, slot).unwrap();
        let slot = ports.get_input::<usize>(0).unwrap();
        assert_eq!(**slot, 5);

        // Add output
        let slot = Slot::<usize>::builder().data(10).build();
        ports.add_output(0, slot).unwrap();
        let slot = ports.get_output::<usize>(0).unwrap();
        assert_eq!(**slot, 10);

        // Check ref count
        assert_eq!(Arc::strong_count(&slot), 2);
    }
}

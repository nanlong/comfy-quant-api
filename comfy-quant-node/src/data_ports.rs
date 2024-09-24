use anyhow::Result;
use std::any::Any;
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Debug)]
pub struct DataPorts {
    inputs: Vec<Option<Box<dyn Any + Send + Sync + 'static>>>,
    outputs: Vec<Option<Box<dyn Any + Send + Sync + 'static>>>,
}

impl DataPorts {
    pub fn new(i: usize, o: usize) -> Self {
        DataPorts {
            inputs: (0..i).map(|_| None).collect(),
            outputs: (0..o).map(|_| None).collect(),
        }
    }
}

impl DataPorts {
    pub fn add_input<T: Send + Sync + 'static>(
        &mut self,
        slot: usize,
        rx: Receiver<T>,
    ) -> Result<()> {
        self.inputs
            .get_mut(slot)
            .ok_or_else(|| anyhow::anyhow!("Invalid input slot"))?
            .replace(Box::new(rx));
        Ok(())
    }

    pub fn add_output<T: Send + Sync + 'static>(
        &mut self,
        slot: usize,
        tx: Sender<T>,
    ) -> Result<()> {
        self.outputs
            .get_mut(slot)
            .ok_or_else(|| anyhow::anyhow!("Invalid output slot"))?
            .replace(Box::new(tx));
        Ok(())
    }

    pub fn get_input<T: Send + Sync + 'static>(&mut self, slot: usize) -> Result<&mut Receiver<T>> {
        let rx = self.inputs[slot]
            .as_mut()
            .ok_or(anyhow::anyhow!("Input slot {} is not connected", slot))?
            .downcast_mut::<Receiver<T>>()
            .ok_or(anyhow::anyhow!("Input slot {} is not connected", slot))?;
        Ok(rx)
    }

    pub fn get_output<T: Send + Sync + 'static>(&self, slot: usize) -> Result<&Sender<T>> {
        let tx = self.outputs[slot]
            .as_ref()
            .ok_or(anyhow::anyhow!("Output slot {} is not connected 1", slot))?
            .downcast_ref::<Sender<T>>()
            .ok_or(anyhow::anyhow!("Output slot {} is not connected 2", slot))?;
        Ok(tx)
    }

    pub fn connection<T: Send + Sync + 'static>(
        &mut self,
        other: &mut Self,
        origin_slot: usize,
        target_slot: usize,
    ) -> Result<()> {
        let tx = self.get_output::<T>(origin_slot)?;
        let rx = tx.subscribe();
        other.add_input(target_slot, rx)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[test]
    fn test_data_ports_new() {
        let ports = DataPorts::new(1, 1);
        assert_eq!(ports.inputs.len(), 1);
        assert_eq!(ports.outputs.len(), 1);
    }

    #[test]
    fn test_data_ports_add_input() {
        let mut ports = DataPorts::new(1, 1);
        let (_tx, rx) = broadcast::channel::<u32>(16);
        ports.add_input(0, rx).unwrap();

        assert!(ports.get_input::<u32>(0).is_ok());
        assert!(ports.get_input::<u64>(0).is_err());
    }

    #[test]
    fn test_data_ports_add_output() {
        let mut ports = DataPorts::new(1, 1);
        let (tx, _rx) = broadcast::channel::<u32>(16);
        ports.add_output(0, tx).unwrap();

        assert!(ports.get_output::<u32>(0).is_ok());
        assert!(ports.get_output::<u64>(0).is_err());
    }

    #[test]
    fn test_data_ports_connection() -> anyhow::Result<()> {
        let mut ports1 = DataPorts::new(1, 1);
        let mut ports2 = DataPorts::new(1, 1);
        let (tx, _rx) = broadcast::channel::<u32>(16);
        ports1.add_output(0, tx)?;
        assert!(ports2.get_input::<u32>(0).is_err());
        ports1.connection::<u32>(&mut ports2, 0, 0)?;
        assert!(ports2.get_input::<u32>(0).is_ok());
        Ok(())
    }
}

use anyhow::Result;
use std::{any::Any, collections::HashMap, fmt, sync::Arc, time::Duration};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time,
};

type AnyMap = HashMap<String, Box<dyn AnyClone + Send + Sync>>;

#[derive(Clone, Default)]
pub struct Assigns {
    map: Option<Box<AnyMap>>,
}

impl Assigns {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    ///
    /// # Example
    ///
    /// ```
    /// # use axum_ws::Assigns;
    /// let mut assigns = Assigns::new();
    /// assert!(assigns.insert("my_key", 5i32).is_none());
    /// assert_eq!(assigns.insert("my_key", 9i32), Some(5i32));
    /// ```
    pub fn insert<T>(&mut self, key: impl Into<String>, val: T) -> Option<T>
    where
        T: AnyClone + Send + Sync + 'static,
    {
        self.map
            .get_or_insert_with(Box::default)
            .insert(key.into(), Box::new(val))
            .and_then(|boxed| boxed.into_any().downcast().ok().map(|boxed| *boxed))
    }

    ///
    /// # Example
    ///
    /// ```
    /// # use axum_ws::Assigns;
    /// let mut assigns = Assigns::new();
    /// assert!(assigns.get::<i32>("my_key").is_none());
    /// assigns.insert("my_key", 5i32);
    ///
    /// assert_eq!(assigns.get::<i32>("my_key"), Some(&5i32));
    /// ```
    pub fn get<T>(&self, key: impl AsRef<str>) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .as_ref()
            .and_then(|map| map.get(key.as_ref()))
            .and_then(|boxed| (**boxed).as_any().downcast_ref())
    }

    pub fn get_mut<T>(&mut self, key: impl AsRef<str>) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .as_mut()
            .and_then(|map| map.get_mut(key.as_ref()))
            .and_then(|boxed| (**boxed).as_any_mut().downcast_mut())
    }

    pub fn get_or_insert<T>(&mut self, key: impl Into<String>, value: T) -> &mut T
    where
        T: AnyClone + Send + Sync + 'static,
    {
        self.get_or_insert_with(key, || value)
    }

    pub fn get_or_insert_with<T, F>(&mut self, key: impl Into<String>, f: F) -> &mut T
    where
        T: AnyClone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let out = self
            .map
            .get_or_insert_with(Box::default)
            .entry(key.into())
            .or_insert_with(|| Box::new(f()));

        (**out).as_any_mut().downcast_mut().unwrap()
    }

    pub fn get_or_insert_default<T>(&mut self, key: impl Into<String>) -> &mut T
    where
        T: Default + AnyClone + Send + Sync + 'static,
    {
        self.get_or_insert_with(key, T::default)
    }

    pub fn remove<T>(&mut self, key: impl AsRef<str>) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .as_mut()
            .and_then(|map| map.remove(key.as_ref()))
            .and_then(|boxed| boxed.into_any().downcast().ok().map(|boxed| *boxed))
    }

    #[inline]
    pub fn clear(&mut self) {
        if let Some(ref mut map) = self.map {
            map.clear();
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.as_ref().map_or(true, |map| map.is_empty())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.as_ref().map_or(0, |map| map.len())
    }

    pub fn extend(&mut self, other: Self) {
        if let Some(other) = other.map {
            if let Some(map) = &mut self.map {
                map.extend(*other);
            } else {
                self.map = Some(other);
            }
        }
    }
}

impl fmt::Debug for Assigns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Assigns").finish()
    }
}

pub trait AnyClone: Any + fmt::Debug {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Clone + fmt::Debug + Send + Sync + 'static> AnyClone for T {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Clone for Box<dyn AnyClone + Send + Sync> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

// trait Slot {
//     type Output;
//     async fn get_data(&self) -> Result<Self::Output>;
//     async fn subscribe(&self) -> Result<Receiver<Self::Output>>;
// }

#[derive(Debug, Clone)]
struct Slot<T: fmt::Debug + Clone + Send + Sync + 'static> {
    data: Option<T>,
    tx: Sender<T>,
}

impl<T: fmt::Debug + Clone + Send + Sync + 'static> Slot<T> {
    pub fn new(data: Option<T>) -> Self {
        let (tx, _) = broadcast::channel::<T>(16);
        Self { data, tx }
    }

    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn send(&self, data: T) -> Result<()> {
        self.tx.send(data)?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<T> {
        self.tx.subscribe()
    }
}

trait Node {
    // async fn connect(
    //     &self,
    //     target: &mut Self,
    //     origin_slot: usize,
    //     target_slot: usize,
    // ) -> Result<()>;
    fn input<T: fmt::Debug + Clone + Send + Sync + 'static>(
        &self,
        slot: usize,
    ) -> Result<Arc<Slot<T>>>;
    // async fn execute(&self) -> Result<()>;
}

#[derive(Clone, Debug)]
struct ExchangeInfo {
    name: String,
    market: String,
    base_currency: String,
    quote_currency: String,
}

#[derive(Debug, Clone)]
struct Ticker {
    timestamp: i64,
    price: f64,
}

#[derive(Debug)]
pub struct DataPorts {
    // inputs: Vec<Option<Arc<Slot<dyn Any + Send + Sync + 'static>>>>,
    outputs: Assigns,
}

impl DataPorts {
    pub fn new(i: usize, o: usize) -> Self {
        // let mut inputs = Vec::with_capacity(i);
        // inputs.extend((0..i).map(|_| None));

        DataPorts {
            outputs: Assigns::new(),
        }
    }
}

impl DataPorts {
    pub fn add_output<T: fmt::Debug + Clone + Send + Sync + 'static>(
        &mut self,
        slot: usize,
        value: Slot<T>,
    ) -> Result<()>
    where
        T: Send,
    {
        self.outputs.insert(slot.to_string(), Arc::new(value));
        Ok(())
    }

    pub fn get_output<T: fmt::Debug + Clone + Send + Sync + 'static>(
        &self,
        slot: usize,
    ) -> Result<Arc<Slot<T>>> {
        let slot = self
            .outputs
            .get::<Arc<Slot<T>>>(slot.to_string())
            .and_then(|s| Some(Arc::clone(s)))
            .ok_or(anyhow::anyhow!("Output slot {} is not connected", slot))?;

        Ok(slot)
    }
}

struct BinanceSpotTicker {
    data_ports: DataPorts,
}

impl BinanceSpotTicker {
    pub fn new() -> Self {
        let mut data_ports = DataPorts::new(0, 1);

        let exchange_info_slot = Slot::new(Some(ExchangeInfo {
            name: "Binance".to_string(),
            market: "Spot".to_string(),
            base_currency: "BTC".to_string(),
            quote_currency: "USDT".to_string(),
        }));

        data_ports.add_output(0, exchange_info_slot).unwrap();
        data_ports.add_output(1, Slot::new(None::<Ticker>)).unwrap();

        Self { data_ports }
    }

    fn execute(&self) -> Result<()> {
        let exchange_info_slot = self.data_ports.get_output::<ExchangeInfo>(0)?;
        let exchange_info = exchange_info_slot.data().unwrap().clone();

        tokio::spawn(async move {
            loop {
                time::sleep(Duration::from_secs(1)).await;
                exchange_info_slot.send(exchange_info.clone()).unwrap();
            }
        });

        Ok(())
    }
}

impl Node for BinanceSpotTicker {
    fn input<T: fmt::Debug + Clone + Send + Sync + 'static>(
        &self,
        slot: usize,
    ) -> Result<Arc<Slot<T>>> {
        self.data_ports.get_output(slot)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let binance_spot_ticker = BinanceSpotTicker::new();
    binance_spot_ticker.execute()?;

    let exchange_info_slot = binance_spot_ticker.input::<ExchangeInfo>(0)?;
    let exchange_info = exchange_info_slot.data();
    println!("{:?}", exchange_info);

    let mut rx = exchange_info_slot.subscribe();

    while let Ok(exchange_info) = rx.recv().await {
        println!("{:?}", exchange_info);
    }

    Ok(())
}

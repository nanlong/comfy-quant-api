use crate::base::traits::spot_order_client::{
    AccountInformation, Balance, Order, OrderSide, OrderStatus, OrderType, SpotOrderClient,
};
use anyhow::Result;
use std::{cell::Cell, collections::HashMap};

pub struct MockSpotClient {
    pub(crate) assets: HashMap<String, Balance>,
    pub(crate) order_id: Cell<u64>,
    pub(crate) order_history: Vec<Order>,
}

impl MockSpotClient {
    pub fn new() -> Self {
        MockSpotClient {
            assets: HashMap::default(),
            order_id: Cell::new(0),
            order_history: Vec::new(),
        }
    }

    pub fn init_assets(&mut self, assets: Vec<Balance>) {
        assets.into_iter().for_each(|asset| {
            self.assets.insert(asset.asset.clone(), asset);
        });
    }

    fn add_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let balance = self.assets.entry(asset.to_string()).or_insert(
            Balance::builder()
                .asset(asset)
                .free("0")
                .locked("0")
                .build(),
        );

        let free = balance.free.parse::<f64>()?;

        balance.free = (free + amount).to_string();

        Ok(())
    }

    fn sub_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let balance = self
            .assets
            .get_mut(asset)
            .ok_or(anyhow::anyhow!("Asset not found"))?;

        let free = balance.free.parse::<f64>()?;

        if free < amount {
            return Err(anyhow::anyhow!("Insufficient free balance"));
        }

        balance.free = (free - amount).to_string();

        Ok(())
    }

    fn lock_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let balance = self
            .assets
            .get_mut(asset)
            .ok_or(anyhow::anyhow!("Asset not found"))?;

        let free = balance.free.parse::<f64>()?;
        let locked = balance.locked.parse::<f64>()?;

        if free < amount {
            return Err(anyhow::anyhow!("Insufficient free balance"));
        }

        balance.free = (free - amount).to_string();
        balance.locked = (locked + amount).to_string();

        Ok(())
    }

    fn unlock_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let balance = self
            .assets
            .get_mut(asset)
            .ok_or(anyhow::anyhow!("Asset not found"))?;

        let free = balance.free.parse::<f64>()?;
        let locked = balance.locked.parse::<f64>()?;

        if locked < amount {
            return Err(anyhow::anyhow!("Insufficient locked balance"));
        }

        balance.free = (free + amount).to_string();
        balance.locked = (locked - amount).to_string();

        Ok(())
    }
}

impl SpotOrderClient for MockSpotClient {
    fn get_account(&self) -> Result<AccountInformation> {
        Ok(AccountInformation::builder()
            .maker_commission(0.001)
            .taker_commission(0.001)
            .build())
    }

    fn get_balance(&self, asset: &str) -> Result<Balance> {
        match self.assets.get(asset) {
            Some(balance) => Ok(balance.clone()),
            None => Ok(Balance::builder()
                .asset(asset)
                .free("0")
                .locked("0")
                .build()),
        }
    }

    fn get_order(&self, order_id: &str) -> Result<Order> {
        let order = self
            .order_history
            .iter()
            .find(|order| order.order_id == order_id)
            .ok_or(anyhow::anyhow!("Order not found"))?
            .clone();

        Ok(order)
    }

    fn market_buy(&self, _symbol: &str, _qty: f64) -> Result<Order> {
        unimplemented!()
    }

    fn market_sell(&self, _symbol: &str, _qty: f64) -> Result<Order> {
        unimplemented!()
    }

    fn limit_buy(&self, symbol: &str, qty: f64, price: f64) -> Result<Order> {
        self.order_id.set(self.order_id.get() + 1);

        let order = Order::builder()
            .symbol(symbol)
            .order_id(self.order_id.get().to_string())
            .price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .order_type(OrderType::Limit)
            .order_side(OrderSide::Buy)
            .order_status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }

    fn limit_sell(&self, symbol: &str, qty: f64, price: f64) -> Result<Order> {
        self.order_id.set(self.order_id.get() + 1);

        let order = Order::builder()
            .symbol(symbol)
            .order_id(self.order_id.get().to_string())
            .price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .order_type(OrderType::Limit)
            .order_side(OrderSide::Sell)
            .order_status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }
}

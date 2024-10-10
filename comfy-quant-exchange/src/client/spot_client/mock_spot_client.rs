use super::{
    base::{AccountInformation, Balance, Order, OrderSide, OrderStatus, OrderType},
    SpotExchangeClient,
};
use anyhow::Result;
use bon::bon;
use std::{cell::Cell, collections::HashMap};

#[derive(Debug)]
pub struct MockSpotClient {
    assets: HashMap<String, Balance>,
    commissions: Option<f64>,
    order_id: Cell<u64>,
    order_history: Vec<Order>,
}

#[bon]
#[allow(unused)]
impl MockSpotClient {
    #[builder]
    pub fn new(assets: Vec<(String, f64)>, commissions: Option<f64>) -> Self {
        let assets = assets
            .into_iter()
            .map(|(asset, amount)| {
                let balance = Balance::builder()
                    .asset(asset.clone())
                    .free(amount.to_string())
                    .locked("0")
                    .build();

                (asset, balance)
            })
            .collect();

        MockSpotClient {
            assets,
            commissions,
            order_id: Cell::new(0),
            order_history: Vec::new(),
        }
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

impl SpotExchangeClient for MockSpotClient {
    async fn get_account(&self) -> Result<AccountInformation> {
        Ok(AccountInformation::builder()
            .maker_commission(self.commissions.unwrap_or(0.001) as f32)
            .taker_commission(self.commissions.unwrap_or(0.001) as f32)
            .build())
    }

    async fn get_balance(&self, asset: &str) -> Result<Balance> {
        match self.assets.get(asset) {
            Some(balance) => Ok(balance.clone()),
            None => Ok(Balance::builder()
                .asset(asset)
                .free("0")
                .locked("0")
                .build()),
        }
    }

    async fn get_order(&self, order_id: &str) -> Result<Order> {
        let order = self
            .order_history
            .iter()
            .find(|order| order.order_id == order_id)
            .ok_or(anyhow::anyhow!("Order not found"))?
            .clone();

        Ok(order)
    }

    async fn market_buy(&self, _symbol: &str, _qty: f64) -> Result<Order> {
        unimplemented!()
    }

    async fn market_sell(&self, _symbol: &str, _qty: f64) -> Result<Order> {
        unimplemented!()
    }

    async fn limit_buy(&self, symbol: &str, qty: f64, price: f64) -> Result<Order> {
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

    async fn limit_sell(&self, symbol: &str, qty: f64, price: f64) -> Result<Order> {
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

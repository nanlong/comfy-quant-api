use super::base::{
    AccountInformation, Balance, Order, OrderSide, OrderStatus, OrderType, SymbolInformation,
};
use crate::client::spot_client_kind::SpotClientExecutable;
use anyhow::Result;
use bon::bon;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct BacktestSpotClientData {
    assets: HashMap<String, Balance>,
    commissions: Option<f64>,
    order_id: u64,
    order_history: Vec<Order>,
}

#[derive(Debug, Clone)]
pub struct BacktestSpotClient {
    data: Arc<Mutex<BacktestSpotClientData>>,
}

#[bon]
#[allow(unused)]
impl BacktestSpotClient {
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

        let data = BacktestSpotClientData {
            assets,
            commissions,
            order_id: 0,
            order_history: Vec::new(),
        };

        BacktestSpotClient {
            data: Arc::new(Mutex::new(data)),
        }
    }

    async fn add_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data.assets.entry(asset.to_string()).or_insert(
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

    async fn sub_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data
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

    async fn lock_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data
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

    async fn unlock_asset(&mut self, asset: &str, amount: f64) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data
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

    fn to_symbol(base_asset: &str, quote_asset: &str) -> String {
        format!(
            "{}{}",
            base_asset.to_uppercase(),
            quote_asset.to_uppercase()
        )
    }
}

impl SpotClientExecutable for BacktestSpotClient {
    async fn get_account(&self) -> Result<AccountInformation> {
        let data = self.data.lock().await;

        Ok(AccountInformation::builder()
            .maker_commission(data.commissions.unwrap_or(0.001))
            .taker_commission(data.commissions.unwrap_or(0.001))
            .build())
    }

    async fn get_symbol_info(
        &self,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<SymbolInformation> {
        let symbol = format!(
            "{}{}",
            base_asset.to_uppercase(),
            quote_asset.to_uppercase()
        );

        Ok(SymbolInformation::builder()
            .symbol(symbol)
            .base_asset(base_asset.to_string())
            .quote_asset(quote_asset.to_string())
            .base_asset_precision(3)
            .quote_asset_precision(3)
            .build())
    }

    async fn get_balance(&self, asset: &str) -> Result<Balance> {
        let data = self.data.lock().await;

        match data.assets.get(asset) {
            Some(balance) => Ok(balance.clone()),
            None => Ok(Balance::builder()
                .asset(asset)
                .free("0")
                .locked("0")
                .build()),
        }
    }

    async fn get_order(&self, order_id: &str) -> Result<Order> {
        let data = self.data.lock().await;

        let order = data
            .order_history
            .iter()
            .find(|order| order.id == order_id)
            .ok_or(anyhow::anyhow!("Order not found"))?
            .clone();

        Ok(order)
    }

    async fn market_buy(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let symbol = Self::to_symbol(base_asset, quote_asset);
        let mut data = self.data.lock().await;
        data.order_id += 1;

        let order = Order::builder()
            .symbol(symbol)
            .id(data.order_id.to_string())
            .price("0".to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .r#type(OrderType::Limit)
            .side(OrderSide::Buy)
            .status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }

    async fn market_sell(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let symbol = Self::to_symbol(base_asset, quote_asset);
        let mut data = self.data.lock().await;
        data.order_id += 1;

        let order = Order::builder()
            .symbol(symbol)
            .id(data.order_id.to_string())
            .price("0".to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .r#type(OrderType::Limit)
            .side(OrderSide::Sell)
            .status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }

    async fn limit_buy(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order> {
        let symbol = Self::to_symbol(base_asset, quote_asset);
        let mut data = self.data.lock().await;
        data.order_id += 1;

        let order = Order::builder()
            .symbol(symbol)
            .id(data.order_id.to_string())
            .price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .r#type(OrderType::Limit)
            .side(OrderSide::Buy)
            .status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }

    async fn limit_sell(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order> {
        let symbol = Self::to_symbol(base_asset, quote_asset);
        let mut data = self.data.lock().await;
        data.order_id += 1;

        let order = Order::builder()
            .symbol(symbol)
            .id(data.order_id.to_string())
            .price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .r#type(OrderType::Limit)
            .side(OrderSide::Sell)
            .status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }
}

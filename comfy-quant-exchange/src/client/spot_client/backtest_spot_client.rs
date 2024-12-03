use super::base::{
    AccountInformation, Balance, Exchange, Order, OrderSide, OrderStatus, OrderType,
    SymbolInformation, SymbolPrice, BACKTEST_EXCHANGE_NAME,
};
use crate::{client::spot_client_kind::SpotClientExecutable, store::PriceStore};
use anyhow::Result;
use async_lock::RwLock;
use bon::bon;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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
    data: Arc<Mutex<BacktestSpotClientData>>, // 必须使用内部可变性和Sync
    price_store: Arc<RwLock<PriceStore>>,     // 价格存储
}

#[bon]
#[allow(unused)]
impl BacktestSpotClient {
    #[builder]
    pub fn new(
        #[builder(into)] assets: Vec<(String, f64)>,
        commissions: Option<f64>,
        price_store: Arc<RwLock<PriceStore>>,
    ) -> Self {
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

        let data = Arc::new(Mutex::new(BacktestSpotClientData {
            assets,
            commissions,
            order_id: 0,
            order_history: Vec::new(),
        }));

        BacktestSpotClient { data, price_store }
    }

    async fn price(&self, symbol: &str) -> Decimal {
        self.price_store
            .read()
            .await
            .price(BACKTEST_EXCHANGE_NAME, "spot", symbol)
            .unwrap_or(dec!(0))
    }

    async fn add_asset(&mut self, asset: &str, amount: Decimal) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data.assets.entry(asset.to_string()).or_insert(
            Balance::builder()
                .asset(asset)
                .free("0")
                .locked("0")
                .build(),
        );

        let free = balance.free.parse::<Decimal>()?;

        balance.free = (free + amount).to_string();

        Ok(())
    }

    async fn sub_asset(&mut self, asset: &str, amount: Decimal) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data
            .assets
            .get_mut(asset)
            .ok_or(anyhow::anyhow!("Asset not found"))?;

        let free = balance.free.parse::<Decimal>()?;

        if free < amount {
            return Err(anyhow::anyhow!("Insufficient free balance"));
        }

        balance.free = (free - amount).to_string();

        Ok(())
    }

    async fn lock_asset(&mut self, asset: &str, amount: Decimal) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data
            .assets
            .get_mut(asset)
            .ok_or(anyhow::anyhow!("Asset not found"))?;

        let free = balance.free.parse::<Decimal>()?;
        let locked = balance.locked.parse::<Decimal>()?;

        if free < amount {
            return Err(anyhow::anyhow!("Insufficient free balance"));
        }

        balance.free = (free - amount).to_string();
        balance.locked = (locked + amount).to_string();

        Ok(())
    }

    async fn unlock_asset(&mut self, asset: &str, amount: Decimal) -> Result<()> {
        let mut data = self.data.lock().await;

        let balance = data
            .assets
            .get_mut(asset)
            .ok_or(anyhow::anyhow!("Asset not found"))?;

        let free = balance.free.parse::<Decimal>()?;
        let locked = balance.locked.parse::<Decimal>()?;

        if locked < amount {
            return Err(anyhow::anyhow!("Insufficient locked balance"));
        }

        balance.free = (free + amount).to_string();
        balance.locked = (locked - amount).to_string();

        Ok(())
    }
}

impl SpotClientExecutable for BacktestSpotClient {
    fn exchange(&self) -> Exchange {
        Exchange::new(BACKTEST_EXCHANGE_NAME)
    }

    fn symbol(&self, base_asset: &str, quote_asset: &str) -> String {
        format!(
            "{}{}",
            base_asset.to_uppercase(),
            quote_asset.to_uppercase()
        )
    }

    async fn get_account(&self) -> Result<AccountInformation> {
        let data = self.data.lock().await;
        let commissions = data.commissions.unwrap_or(0.001);
        let commission_rate = commissions.try_into()?;

        Ok(AccountInformation::builder()
            .maker_commission_rate(commission_rate)
            .taker_commission_rate(commission_rate)
            .can_trade(true)
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
            .base_asset(base_asset)
            .quote_asset(quote_asset)
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

    async fn get_order(
        &self,
        _base_asset: &str,
        _quote_asset: &str,
        order_id: &str,
    ) -> Result<Order> {
        let data = self.data.lock().await;

        let order = data
            .order_history
            .iter()
            .find(|order| order.order_id == order_id)
            .ok_or(anyhow::anyhow!("Order not found"))?
            .clone();

        Ok(order)
    }

    async fn market_buy(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let symbol = self.symbol(base_asset, quote_asset);
        let qty = Decimal::try_from(qty)?;
        let price = self.price(&symbol).await;
        let mut data = self.data.lock().await;

        data.order_id += 1;

        let order = Order::builder()
            .exchange(BACKTEST_EXCHANGE_NAME)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .symbol(symbol)
            .order_id(data.order_id.to_string())
            .price(price.to_string())
            .avg_price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty(qty.to_string())
            .cumulative_quote_qty((qty * price).to_string())
            .order_type(OrderType::Market)
            .order_side(OrderSide::Buy)
            .order_status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }

    async fn market_sell(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let symbol = self.symbol(base_asset, quote_asset);
        let qty = Decimal::try_from(qty)?;
        let price = self.price(&symbol).await;
        let mut data = self.data.lock().await;

        data.order_id += 1;

        let order = Order::builder()
            .exchange(BACKTEST_EXCHANGE_NAME)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .symbol(symbol)
            .order_id(data.order_id.to_string())
            .price(price.to_string())
            .avg_price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty(qty.to_string())
            .cumulative_quote_qty((qty * price).to_string())
            .order_type(OrderType::Market)
            .order_side(OrderSide::Sell)
            .order_status(OrderStatus::Filled)
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
        let symbol = self.symbol(base_asset, quote_asset);
        let mut data = self.data.lock().await;
        data.order_id += 1;

        let order = Order::builder()
            .exchange(BACKTEST_EXCHANGE_NAME)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .symbol(symbol)
            .order_id(data.order_id.to_string())
            .price(price.to_string())
            .avg_price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .cumulative_quote_qty("0")
            .order_type(OrderType::Limit)
            .order_side(OrderSide::Buy)
            .order_status(OrderStatus::Filled)
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
        let symbol = self.symbol(base_asset, quote_asset);
        let mut data = self.data.lock().await;
        data.order_id += 1;

        let order = Order::builder()
            .exchange(BACKTEST_EXCHANGE_NAME)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .symbol(symbol)
            .order_id(data.order_id.to_string())
            .price(price.to_string())
            .avg_price(price.to_string())
            .orig_qty(qty.to_string())
            .executed_qty("0")
            .cumulative_quote_qty("0")
            .order_type(OrderType::Limit)
            .order_side(OrderSide::Sell)
            .order_status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }

    async fn get_price(&self, base_asset: &str, quote_asset: &str) -> Result<SymbolPrice> {
        let symbol = self.symbol(base_asset, quote_asset);
        let price = self.price(&symbol).await;
        Ok(SymbolPrice::builder().symbol(symbol).price(price).build())
    }
}

use crate::{
    node_core::{
        AssetAmount, NodeContext, NodeExecutable, NodeInfo, NodeInfra, NodePort, NodeStats, Port,
        SpotClientService, SpotTradeable, TradeStats,
    },
    node_io::{SpotPairInfo, TickStream},
    stats::SpotStats,
    workflow::Node,
};
use anyhow::{anyhow, Result};
use bon::{bon, Builder};
use comfy_quant_base::{Exchange, Market, Symbol};
use comfy_quant_exchange::client::{
    spot_client::base::{Order, OrderSide},
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal, MathematicalOps,
};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
};

/// 网格交易
/// inputs:
///     0: SpotPairInfo
///     1: SpotClientKind
///     2: TickStream
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct SpotGrid {
    params: Params,
    store: RuntimeStore,
    infra: NodeInfra,
}

impl SpotGrid {
    pub(crate) fn try_new(node: Node) -> Result<Self> {
        let params = Params::try_from(&node)?;
        let store = RuntimeStore::try_from(&node)?;
        let infra = NodeInfra::new(node);

        Ok(Self {
            params,
            store,
            infra,
        })
    }

    pub(crate) async fn initialize(
        &mut self,
        pair_info: &SpotPairInfo,
        client: &SpotClientKind,
        tick_stream: &TickStream,
    ) -> Result<()> {
        // 获取初始化价格
        let (_, _, tick) = tick_stream.subscribe().recv_async().await?;
        let initial_price = tick.price;

        // 如果已经初始化，则跳过
        if self.store.initialized {
            return Ok(());
        }

        // 创建客户端服务
        let mut spot_client_service = SpotClientService::builder()
            .client(client)
            .retry_max_retries(3)
            .retry_wait_secs(3)
            .timeout_secs(10)
            .build();

        // 获取账户信息
        let account = spot_client_service.get_account().await?;

        // 获取账户余额
        let balance = spot_client_service
            .get_balance(&pair_info.quote_asset)
            .await?;

        // 获取交易对信息
        let symbol_info = spot_client_service
            .get_symbol_info(&pair_info.base_asset, &pair_info.quote_asset)
            .await?;

        // 获取平台名称
        let exchange = spot_client_service.exchange().await?;

        // 检查账户余额是否充足
        if balance.free.parse::<Decimal>()? < self.params.investment {
            anyhow::bail!("Insufficient free balance");
        }

        // 初始化统计信息
        let symbol = client.symbol(&pair_info.base_asset, &pair_info.quote_asset);

        // 初始化统计信息
        self.store.stats.initialize(
            &exchange,
            &symbol,
            &pair_info.base_asset,
            &pair_info.quote_asset,
        );

        // 初始化账户余额
        self.store
            .stats
            .initialize_balance(
                self.infra.node_context()?,
                &exchange,
                &symbol,
                &dec!(0),
                &self.params.investment,
                &initial_price,
            )
            .await?;

        // 计算网格价格
        let grid_prices = calc_grid_prices(
            &self.params.mode,
            self.params.lower_price,
            self.params.upper_price,
            self.params.grid_rows,
            symbol_info.quote_asset_precision,
        );

        // 获取当前价格
        let (_, _, tick) = tick_stream.subscribe().recv_async().await?;

        // 创建网格
        let grid = Grid::builder()
            .exchange(exchange)
            .investment(self.params.investment)
            .grid_prices(grid_prices)
            .current_price(tick.price)
            .base_asset_precision(symbol_info.base_asset_precision)
            .quote_asset_precision(symbol_info.quote_asset_precision)
            .commission_rate(account.taker_commission_rate)
            .build();

        self.store.grid = Some(grid);

        // 初始化完成
        self.store.initialized = true;

        Ok(())
    }

    fn grid(&self) -> Result<&Grid> {
        self.store
            .grid
            .as_ref()
            .ok_or_else(|| anyhow!("SpotGrid grid not initializer"))
    }

    fn grid_mut(&mut self) -> Result<&mut Grid> {
        self.store
            .grid
            .as_mut()
            .ok_or_else(|| anyhow!("SpotGrid grid not initializer"))
    }

    fn exchange_pair_symbol(&self) -> Result<(Exchange, SpotPairInfo, Symbol)> {
        let port = self.port();
        let client = port.input::<SpotClientKind>(1)?;
        let pair_info = port.input::<SpotPairInfo>(0)?;

        let exchange = client.exchange();
        let pair_info = (**pair_info).clone();
        let symbol = client.symbol(&pair_info.base_asset, &pair_info.quote_asset);

        Ok((exchange, pair_info, symbol))
    }
}

impl NodePort for SpotGrid {
    fn port(&self) -> &Port {
        &self.infra.port
    }

    fn port_mut(&mut self) -> &mut Port {
        &mut self.infra.port
    }
}

impl NodeStats for SpotGrid {
    fn spot_stats(&self) -> Option<&SpotStats> {
        Some(&self.store.stats)
    }

    fn spot_stats_mut(&mut self) -> Option<&mut SpotStats> {
        Some(&mut self.store.stats)
    }
}

impl NodeInfo for SpotGrid {
    fn node_context(&self) -> Result<NodeContext> {
        self.infra.node_context()
    }
}

// 节点执行
impl NodeExecutable for SpotGrid {
    async fn execute(&mut self) -> Result<()> {
        // 等待其他节点
        self.infra.workflow_context()?.wait().await;

        // 获取输入
        let port = self.port();
        let pair_info = port.input::<SpotPairInfo>(0)?;
        let client = port.input::<SpotClientKind>(1)?;
        let tick_stream = port.input::<TickStream>(2)?;
        let cloned_params = self.params.clone();
        let rx = tick_stream.subscribe();

        self.initialize(&pair_info, &client, &tick_stream).await?;

        self.grid()?.start();

        while let Ok((_, _, tick)) = rx.recv_async().await {
            let Some(signal) = self
                .grid_mut()?
                .evaluate_with_price(&cloned_params, tick.price)
            else {
                continue;
            };

            match signal {
                // 买入
                TradeSignal::Buy { quantity, .. } => {
                    let order_result = self
                        .market_buy(
                            &client,
                            &pair_info.base_asset,
                            &pair_info.quote_asset,
                            quantity
                                .to_f64()
                                .ok_or_else(|| anyhow!("Failed to convert quantity to f64"))?,
                        )
                        .await;

                    match order_result {
                        Ok(order) => {
                            self.grid_mut()?.update_with_order(&signal, &order);
                            tracing::info!("SpotGrid buy order: {:?}", order);
                        }
                        Err(e) => {
                            self.grid()?.unlock();
                            tracing::error!("SpotGrid buy order failed: {}", e);
                        }
                    }
                }

                // 卖出
                TradeSignal::Sell { quantity, .. } => {
                    let order_result = self
                        .market_sell(
                            &client,
                            &pair_info.base_asset,
                            &pair_info.quote_asset,
                            quantity
                                .to_f64()
                                .ok_or_else(|| anyhow!("Failed to convert quantity to f64"))?,
                        )
                        .await;

                    match order_result {
                        Ok(order) => {
                            self.grid_mut()?.update_with_order(&signal, &order);
                            tracing::info!("SpotGrid sell order: {:?}", order);
                        }
                        Err(e) => {
                            self.grid()?.unlock();
                            tracing::error!("SpotGrid sell order failed: {}", e);
                        }
                    }
                }

                // 止损
                TradeSignal::StopLoss { sell_all_on_stop } => {
                    if !sell_all_on_stop {
                        continue;
                    }

                    let Ok(balance) = client.get_balance(&pair_info.base_asset).await else {
                        self.grid()?.unlock();
                        continue;
                    };

                    let order_result = self
                        .market_sell(
                            &client,
                            &pair_info.base_asset,
                            &pair_info.quote_asset,
                            balance.free.parse::<f64>()?,
                        )
                        .await;

                    match order_result {
                        Ok(order) => {
                            self.grid_mut()?.update_with_order(&signal, &order);
                            self.grid()?.stop();
                            tracing::info!("SpotGrid sell all order: {:?}", order);
                        }
                        Err(e) => {
                            self.grid()?.unlock();
                            tracing::error!("SpotGrid sell all order failed: {}", e);
                        }
                    }
                }

                // 止盈
                TradeSignal::TakeProfit => {
                    let Ok(balance) = client.get_balance(&pair_info.base_asset).await else {
                        self.grid()?.unlock();
                        continue;
                    };

                    let order_result = self
                        .market_sell(
                            &client,
                            &pair_info.base_asset,
                            &pair_info.quote_asset,
                            balance.free.parse::<f64>()?,
                        )
                        .await;

                    match order_result {
                        Ok(order) => {
                            self.grid_mut()?.update_with_order(&signal, &order);
                            self.grid()?.stop();
                            tracing::info!("SpotGrid take profit order: {:?}", order);
                        }
                        Err(e) => {
                            self.grid()?.unlock();
                            tracing::error!("SpotGrid take profit order failed: {}", e);
                        }
                    }
                }
            }

            // 更新统计信息
            self.update_spot_stats_with_tick(
                &client.exchange(),
                &client.symbol(&pair_info.base_asset, &pair_info.quote_asset),
                &tick,
            )
            .await?;
        }

        Ok(())
    }
}

impl TradeStats for SpotGrid {
    async fn initial_capital(&self) -> Result<AssetAmount> {
        let ctx = self.infra.workflow_context()?;
        let (exchange, pair, symbol) = self.exchange_pair_symbol()?;
        let quote_asset = ctx.quote_asset().await;
        let exchange_rate = ctx.exchange_rate(&pair.quote_asset, &quote_asset).await?;
        let stats = self.spot_stats_data(&exchange, &symbol)?;
        let price = self.infra.price(&exchange, Market::Spot, &symbol).await?;
        let capital = stats.initial_base_balance * price + stats.initial_base_balance;

        Ok(AssetAmount::new(
            quote_asset,
            capital * exchange_rate.rate(),
        ))
    }

    async fn realized_pnl(&self) -> Result<AssetAmount> {
        let ctx = self.infra.workflow_context()?;
        let (exchange, pair, symbol) = self.exchange_pair_symbol()?;
        let quote_asset = ctx.quote_asset().await;
        let exchange_rate = ctx.exchange_rate(&pair.quote_asset, &quote_asset).await?;
        let stats = self.spot_stats_data(exchange, symbol)?;

        Ok(AssetAmount::new(
            quote_asset,
            stats.base.realized_pnl * exchange_rate.rate(),
        ))
    }

    async fn unrealized_pnl(&self) -> Result<AssetAmount> {
        let ctx = self.infra.workflow_context()?;
        let (exchange, pair, symbol) = self.exchange_pair_symbol()?;
        let quote_asset = ctx.quote_asset().await;
        let exchange_rate = ctx.exchange_rate(&pair.quote_asset, &quote_asset).await?;
        let stats = self.spot_stats_data(&exchange, &symbol)?;
        let price = self.infra.price(&exchange, Market::Spot, &symbol).await?;
        let maker_commission_rate = Decimal::ONE - stats.base.maker_commission_rate;
        let cost = stats.base_asset_balance * stats.avg_price;
        let maybe_sell = stats.base_asset_balance * price * maker_commission_rate;
        let unrealized_pnl = maybe_sell - cost;

        Ok(AssetAmount::new(
            quote_asset,
            unrealized_pnl * exchange_rate.rate(),
        ))
    }

    async fn total_pnl(&self) -> Result<AssetAmount> {
        let ctx = self.infra.workflow_context()?;
        let quote_asset = ctx.quote_asset().await;
        let realized_pnl = self.realized_pnl().await?;
        let unrealized_pnl = self.unrealized_pnl().await?;

        Ok(AssetAmount::new(
            quote_asset,
            realized_pnl.value() + unrealized_pnl.value(),
        ))
    }

    async fn running_time(&self) -> Result<u128> {
        Ok(self.infra.workflow_context()?.running_time().await)
    }
}

impl TryFrom<Node> for SpotGrid {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
        SpotGrid::try_new(node)
    }
}

impl TryFrom<&SpotGrid> for Node {
    type Error = anyhow::Error;

    fn try_from(spot_grid: &SpotGrid) -> Result<Self> {
        let mut node = spot_grid.infra.node.clone();
        node.runtime_store = Some(serde_json::to_string(&spot_grid.store)?);
        Ok(node)
    }
}

#[derive(Builder, Serialize, Deserialize, Debug, Clone)]
#[allow(unused)]
pub(crate) struct Params {
    mode: Mode,                     // 网格模式
    lower_price: Decimal,           // 网格下界
    upper_price: Decimal,           // 网格上界
    grid_rows: u64,                 // 网格数量
    investment: Decimal,            // 投资金额
    trigger_price: Option<Decimal>, // 触发价格
    stop_loss: Option<Decimal>,     // 止损价格
    take_profit: Option<Decimal>,   // 止盈价格
    sell_all_on_stop: bool,         // 是否在止损时卖出所有基准币，默认为true
}

impl TryFrom<&Node> for Params {
    type Error = SpotGridError;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        if node.properties.prop_type != "strategy.SpotGrid" {
            return Err(SpotGridError::PropertyTypeMismatch);
        }

        let [mode, lower_price, upper_price, grid_rows, investment, trigger_price, stop_loss, take_profit, sell_all_on_stop] =
            node.properties.params.as_slice()
        else {
            return Err(SpotGridError::ParamsFormatError);
        };

        let mode = mode
            .as_str()
            .and_then(|mode| mode.parse::<Mode>().ok())
            .ok_or(SpotGridError::ModeError)?;

        let lower_price = lower_price
            .as_f64()
            .and_then(Decimal::from_f64)
            .ok_or(SpotGridError::LowerPriceError)?;

        let upper_price = upper_price
            .as_f64()
            .and_then(Decimal::from_f64)
            .ok_or(SpotGridError::UpperPriceError)?;

        let grid_rows = grid_rows.as_u64().ok_or(SpotGridError::GridRowsError)?;

        let investment = investment
            .as_f64()
            .and_then(Decimal::from_f64)
            .ok_or(SpotGridError::InvestmentError)?;

        let trigger_price = trigger_price.as_f64().and_then(Decimal::from_f64);

        let stop_loss = stop_loss.as_f64().and_then(Decimal::from_f64);

        let take_profit = take_profit.as_f64().and_then(Decimal::from_f64);

        let sell_all_on_stop = sell_all_on_stop.as_bool().unwrap_or(true);

        if lower_price >= upper_price {
            return Err(SpotGridError::PriceRangeError);
        }

        if !(2..150).contains(&grid_rows) {
            return Err(SpotGridError::GridRowsError);
        }

        let params = Params::builder()
            .mode(mode)
            .lower_price(lower_price)
            .upper_price(upper_price)
            .grid_rows(grid_rows)
            .investment(investment)
            .maybe_trigger_price(trigger_price)
            .maybe_stop_loss(stop_loss)
            .maybe_take_profit(take_profit)
            .sell_all_on_stop(sell_all_on_stop)
            .build();

        Ok(params)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SpotGridError {
    #[error("Invalid property type, expected 'strategy.SpotGrid'")]
    PropertyTypeMismatch,

    #[error("Invalid parameters format")]
    ParamsFormatError,

    #[error("Invalid mode")]
    ModeError,

    #[error("Invalid lower_price")]
    LowerPriceError,

    #[error("Invalid upper_price")]
    UpperPriceError,

    #[error("Invalid lower_price must be less than upper_price")]
    PriceRangeError,

    #[error("Invalid grid_rows")]
    GridRowsError,

    #[error("Invalid investment")]
    InvestmentError,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RuntimeStore {
    stats: SpotStats,
    grid: Option<Grid>,
    initialized: bool,
}

impl RuntimeStore {
    fn new() -> Self {
        Self {
            stats: SpotStats::new(),
            grid: None,
            initialized: false,
        }
    }
}

impl TryFrom<&Node> for RuntimeStore {
    type Error = anyhow::Error;

    fn try_from(node: &Node) -> Result<Self> {
        if let Some(runtime_store) = &node.runtime_store {
            Ok(serde_json::from_str(runtime_store)?)
        } else {
            Ok(Self::new())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(crate) enum Mode {
    // 等差
    Arithmetic,
    // 等比
    Geometric,
}

impl FromStr for Mode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mode = match s {
            "arithmetic" => Mode::Arithmetic,
            "geometric" => Mode::Geometric,
            _ => anyhow::bail!("Invalid mode: {}", s),
        };

        Ok(mode)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub(crate) struct Grid {
    exchange: String,         // 平台名称
    rows: Vec<GridRow>,       // 网格行
    cursor: usize,            // 当前网格序号
    prev_sell_price: Decimal, // 上一次的卖出价格
    starting: AtomicBool,     // 是否开始
    running: AtomicBool,      // 是否运行
    locked: AtomicBool,       // 是否锁定
}

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub(crate) enum TradeSignal {
    Buy { price: Decimal, quantity: Decimal },  // 买入数量
    Sell { price: Decimal, quantity: Decimal }, // 卖出数量
    StopLoss { sell_all_on_stop: bool },        // 止损
    TakeProfit,                                 // 止盈
}

#[bon]
impl Grid {
    #[builder]
    fn new(
        #[builder(into)] exchange: String, // 平台名称
        investment: Decimal,               // 投资金额
        grid_prices: Vec<Decimal>,         // 网格价格
        current_price: Decimal,            // 当前价格
        base_asset_precision: u32,         // 基础币种小数点位数
        quote_asset_precision: u32,        // 报价币种小数点位数
        commission_rate: Decimal,          // 手续费
    ) -> Self {
        let grid_investment = (investment / (Decimal::from(grid_prices.len()) - dec!(1)))
            .round_dp(quote_asset_precision);

        let rows = grid_prices
            .windows(2)
            .enumerate()
            .map(|(i, w)| {
                let buy_quantity = (grid_investment / w[0]).round_dp(base_asset_precision);
                let sell_quantity =
                    (buy_quantity * (dec!(1) - commission_rate)).round_dp(base_asset_precision);

                GridRow::builder()
                    .index(i)
                    .buy_price(w[0])
                    .buy_quantity(buy_quantity)
                    .sell_price(w[1])
                    .sell_quantity(sell_quantity)
                    .buyed(false)
                    .sold(false)
                    .build()
            })
            .collect::<Vec<_>>();

        let cursor = rows
            .iter()
            .position(|r| r.buy_price <= current_price && r.sell_price >= current_price)
            .unwrap_or(0);

        let prev_sell_price = dec!(0);

        let starting = AtomicBool::new(false);
        let running = AtomicBool::new(false);
        let locked = AtomicBool::new(false);

        Grid {
            exchange,
            rows,
            cursor,
            prev_sell_price,
            starting,
            running,
            locked,
        }
    }

    fn should_buy(&self, price: Decimal, tolerance: Decimal) -> bool {
        let grid_row = self.current_grid_row();

        !grid_row.buyed // 当前格子未买入
            && self.prev_sell_price != grid_row.buy_price // 上一次的卖出价格不等于当前的买入价格
            && price <= grid_row.buy_price
            && price > grid_row.buy_price * (dec!(1) - tolerance)
    }

    fn should_sell(&self, price: Decimal, tolerane: Decimal) -> bool {
        let grid_row = self.current_grid_row();

        grid_row.buyed // 当前格子已买入
            && !grid_row.sold // 当前格子未卖出
            && price >= grid_row.sell_price
            && price < grid_row.sell_price * (dec!(1) + tolerane)
    }

    fn adjust_grid_position(&mut self, price: Decimal) {
        let grid_row = self.current_grid_row();

        // 向下移动一格
        if price < grid_row.buy_price {
            if let Some(lower_grid) = self.lower_grid() {
                let step = (lower_grid.sell_price - lower_grid.buy_price) / dec!(2);
                if price <= grid_row.buy_price - step {
                    self.cursor -= 1;
                }
            }
        }
        // 向上移动一格
        else if price > grid_row.sell_price {
            if let Some(upper_grid) = self.upper_grid() {
                let step = (upper_grid.sell_price - upper_grid.buy_price) / dec!(2);
                if price >= grid_row.sell_price + step {
                    self.cursor += 1;
                }
            }
        }
    }

    /// 根据当前价格，获取交易信号
    fn evaluate_with_price(
        &mut self,
        params: &Params,
        current_price: Decimal,
    ) -> Option<TradeSignal> {
        // 价格浮动比率
        let price_tolerance = dec!(0.005);

        // 如果未运行或锁定，则不进行操作
        if !self.starting.load(Ordering::Relaxed) || self.locked.load(Ordering::Relaxed) {
            return None;
        }

        if !self.running.load(Ordering::Relaxed) {
            // 如果设置了触发价格，则根据触发价格决定是否运行
            if let Some(trigger_price) = params.trigger_price {
                if current_price <= trigger_price {
                    self.running.store(true, Ordering::Relaxed);
                }
            } else {
                self.running.store(true, Ordering::Relaxed);
            }

            return None;
        }

        // 买入
        if self.should_buy(current_price, price_tolerance) {
            let signal = TradeSignal::Buy {
                price: self.current_grid_row().buy_price,
                quantity: self.current_grid_row().buy_quantity,
            };

            self.lock();
            return Some(signal);
        }

        // 卖出
        if self.should_sell(current_price, price_tolerance) {
            let signal = TradeSignal::Sell {
                price: self.current_grid_row().sell_price,
                quantity: self.current_grid_row().sell_quantity,
            };

            self.lock();
            return Some(signal);
        }

        // 止损
        if let Some(stop_loss) = params.stop_loss {
            if current_price <= stop_loss {
                let signal = TradeSignal::StopLoss {
                    sell_all_on_stop: params.sell_all_on_stop,
                };

                self.starting.store(false, Ordering::Relaxed);
                return Some(signal);
            }
        }

        // 止盈
        if let Some(take_profit) = params.take_profit {
            if current_price >= take_profit {
                let signal = TradeSignal::TakeProfit;

                self.starting.store(false, Ordering::Relaxed);
                return Some(signal);
            }
        }

        // 上下移动格子
        self.adjust_grid_position(current_price);

        None
    }

    /// 更新网格状态
    fn update_with_order(&mut self, signal: &TradeSignal, order: &Order) {
        match order.order_side {
            OrderSide::Buy => {
                self.current_grid_row_mut().buyed = true;
                self.current_grid_row_mut().sold = false;
            }
            OrderSide::Sell => {
                if let TradeSignal::Sell { price, .. } = signal {
                    self.prev_sell_price = *price;
                }

                self.current_grid_row_mut().sold = true;
                self.current_grid_row_mut().buyed = false;
            }
        }

        self.unlock();
    }

    fn stop(&self) {
        self.starting.store(false, Ordering::Relaxed);
        self.running.store(false, Ordering::Relaxed);
        self.locked.store(false, Ordering::Relaxed);
    }

    fn start(&self) {
        self.starting.store(true, Ordering::Relaxed);
        self.running.store(false, Ordering::Relaxed);
        self.locked.store(false, Ordering::Relaxed);
    }

    // 锁定
    fn lock(&self) {
        self.locked.store(true, Ordering::Relaxed);
    }

    // 取消锁定
    fn unlock(&self) {
        self.locked.store(false, Ordering::Relaxed);
    }

    /// 获取当前行
    fn current_grid_row(&self) -> &GridRow {
        &self.rows[self.cursor]
    }

    /// 获取当前行可变引用
    fn current_grid_row_mut(&mut self) -> &mut GridRow {
        &mut self.rows[self.cursor]
    }

    /// 上方向的网格
    fn upper_grid(&self) -> Option<&GridRow> {
        self.rows.get(self.cursor + 1)
    }

    /// 下方向的网格
    fn lower_grid(&self) -> Option<&GridRow> {
        if self.cursor > 0 {
            self.rows.get(self.cursor - 1)
        } else {
            None
        }
    }
}

#[derive(Builder, Debug, Serialize, Deserialize)]
#[allow(unused)]
pub(crate) struct GridRow {
    index: usize,           // 网格序号
    buy_price: Decimal,     // 买入价格
    buy_quantity: Decimal,  // 买入数量
    sell_price: Decimal,    // 卖出价格
    sell_quantity: Decimal, // 卖出数量
    buyed: bool,            // 是否已买入
    sold: bool,             // 是否已卖出
}

// 计算网格价格
fn calc_grid_prices(
    mode: &Mode,                // 网格模式
    lower_price: Decimal,       // 网格下界
    upper_price: Decimal,       // 网格上界
    grid_rows: u64,             // 网格数量
    quote_asset_precision: u32, // 小数点位数
) -> Vec<Decimal> {
    match mode {
        Mode::Arithmetic => {
            let step = (upper_price - lower_price) / Decimal::from(grid_rows);
            (0..=grid_rows)
                .map(|i| (lower_price + step * Decimal::from(i)).round_dp(quote_asset_precision))
                .collect()
        }
        Mode::Geometric => {
            let step = (upper_price / lower_price).powf(1. / grid_rows as f64);

            (0..=grid_rows)
                .map(|i| (lower_price * step.powi(i as i64)).round_dp(quote_asset_precision))
                .collect()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[allow(unused)]
enum GridProfitRate {
    Arithmetic {
        min_rate: Decimal,
        max_rate: Decimal,
    },
    Geometric {
        rate: Decimal,
    },
}

// 计算网格的每格利润率
// 参考资料：https://www.binance.com/zh-CN/support/faq/币安现货网格交易的参数说明-688ff6ff08734848915de76a07b953dd
// #[allow(unused)]
// fn calculate_grid_profit(
//     mode: Mode,                // 网格模式
//     lower_price: Decimal,      // 网格下界
//     upper_price: Decimal,      // 网格上界
//     taker_commission: Decimal, // 手续费
//     grid_rows: u64,            // 网格数量
// ) -> GridProfitRate {
//     match mode {
//         Mode::Arithmetic => {
//             let step = (upper_price - lower_price) / grid_rows as Decimal;
//             let max_profit_rate =
//                 (1. - taker_commission) * step / lower_price - 2. * taker_commission;
//             let min_profit_rate = (upper_price * (1. - taker_commission)) / (upper_price - step)
//                 - 1.
//                 - taker_commission;

//             GridProfitRate::Arithmetic {
//                 min_rate: floor_to(min_profit_rate, 4),
//                 max_rate: floor_to(max_profit_rate, 4),
//             }
//         }
//         Mode::Geometric => {
//             let step = (upper_price / lower_price).powf(1.0 / grid_rows as Decimal);
//             let profit_rate = (1. - taker_commission) * step - 1. - taker_commission;

//             GridProfitRate::Geometric {
//                 rate: floor_to(profit_rate, 4),
//             }
//         }
//     }
// }

#[allow(unused)]
fn calculate_minimum_investment(
    min_qty: Decimal,              // 最小交易数量
    min_notional: Option<Decimal>, // 最小名义价值
    upper_price: Decimal,          // 网格上界
    lower_price: Decimal,          // 网格下界
    grid_rows: u32,                // 网格数量
    current_price: Decimal,        // 当前价格
) -> Decimal {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        node_core::ExchangeRateManager,
        workflow::{QuoteAsset, WorkflowContext},
    };
    use async_lock::{Barrier, RwLock};
    use comfy_quant_exchange::client::spot_client::base::{OrderStatus, OrderType};
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test]
    fn test_try_from_node_to_spot_grid(db: PgPool) -> Result<()> {
        let json_str = r#"{"id":4,"type":"交易策略/网格(现货)","pos":[367,125],"size":{"0":210,"1":310},"flags":{},"order":1,"mode":0,"inputs":[{"name":"交易所信息","type":"exchangeData","link":null},{"name":"最新成交价格","type":"tickerStream","link":null},{"name":"账户","type":"account","link":null},{"name":"回测","type":"backtest","link":null}],"properties":{"type":"strategy.SpotGrid","params":["arithmetic",1,1.1,8,1,"","","",true]}}"#;

        let mut node: Node = serde_json::from_str(json_str)?;
        let workflow_context = Arc::new(WorkflowContext::new(
            Arc::new(db),
            Arc::new(RwLock::new(QuoteAsset::new())),
            Arc::new(RwLock::new(ExchangeRateManager::default())),
            Arc::new(RwLock::new(0)),
            Barrier::new(0),
        ));
        node.context = Some(workflow_context);

        let spot_grid = SpotGrid::try_from(node)?;

        assert_eq!(spot_grid.params.mode, Mode::Arithmetic);
        assert_eq!(spot_grid.params.lower_price, Decimal::try_from(1.0)?);
        assert_eq!(spot_grid.params.upper_price, Decimal::try_from(1.1)?);
        assert_eq!(spot_grid.params.grid_rows, 8);
        assert_eq!(spot_grid.params.investment, Decimal::try_from(1.0)?);
        assert_eq!(spot_grid.params.trigger_price, None);
        assert_eq!(spot_grid.params.stop_loss, None);
        assert_eq!(spot_grid.params.take_profit, None);
        assert_eq!(spot_grid.params.sell_all_on_stop, true);

        let node = Node::try_from(&spot_grid)?;
        assert_eq!(
            node.runtime_store,
            Some("{\"stats\":{\"data\":{}},\"grid\":null,\"initialized\":false}".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_calculate_grid_rows() -> Result<()> {
        let grid_prices = calc_grid_prices(&Mode::Arithmetic, dec!(1.0), dec!(1.1), 8, 3);
        assert_eq!(
            grid_prices,
            vec![
                dec!(1.0),
                dec!(1.012),
                dec!(1.025),
                dec!(1.038),
                dec!(1.05),
                dec!(1.062),
                dec!(1.075),
                dec!(1.088),
                dec!(1.1)
            ]
        );

        let grid_prices = calc_grid_prices(&Mode::Geometric, dec!(1.0), dec!(1.1), 8, 3);
        assert_eq!(
            grid_prices,
            vec![
                dec!(1.0),
                dec!(1.012),
                dec!(1.024),
                dec!(1.036),
                dec!(1.049),
                dec!(1.061),
                dec!(1.074),
                dec!(1.087),
                dec!(1.1)
            ]
        );

        let grid_prices = calc_grid_prices(&Mode::Geometric, dec!(4.0), dec!(20.0), 10, 3);
        assert_eq!(
            grid_prices,
            vec![
                dec!(4.0),
                dec!(4.698),
                dec!(5.519),
                dec!(6.483),
                dec!(7.615),
                dec!(8.944),
                dec!(10.506),
                dec!(12.341),
                dec!(14.496),
                dec!(17.027),
                dec!(20.0)
            ]
        );

        let grid_prices = calc_grid_prices(&Mode::Geometric, dec!(4.0), dec!(20.0), 2, 3);
        assert_eq!(grid_prices, vec![dec!(4.0), dec!(8.944), dec!(20.0)]);

        Ok(())
    }

    // #[test]
    // fn test_calculate_grid_profit() -> Result<()> {
    //     let profit =
    //         calculate_grid_profit(Mode::Arithmetic, dec!(4.0), dec!(20.0), dec!(0.001), 10);
    //     assert_eq!(
    //         profit,
    //         GridProfitRate::Arithmetic {
    //             min_rate: 0.0848,
    //             max_rate: 0.3976
    //         }
    //     );

    //     let profit = calculate_grid_profit(Mode::Geometric, 4.0, 20.0, 0.001, 10);
    //     assert_eq!(profit, GridProfitRate::Geometric { rate: 0.1724 });

    //     Ok(())
    // }

    #[test]
    fn test_grid_logic() -> Result<()> {
        let params = Params::builder()
            .mode(Mode::Geometric)
            .lower_price(dec!(4.0))
            .upper_price(dec!(20.0))
            .grid_rows(10)
            .investment(dec!(1000.0))
            .sell_all_on_stop(true)
            .build();

        let base_asset_precision = 2;
        let quote_asset_precision = 3;
        let commission = dec!(0.001);

        let grid_prices = calc_grid_prices(
            &params.mode,
            params.lower_price,
            params.upper_price,
            params.grid_rows,
            quote_asset_precision,
        );

        let mut grid = Grid::builder()
            .exchange("Test")
            .investment(params.investment)
            .grid_prices(grid_prices)
            .base_asset_precision(base_asset_precision)
            .quote_asset_precision(quote_asset_precision)
            .current_price(dec!(4.25))
            .commission_rate(commission)
            .build();

        grid.start();

        assert_eq!(grid.rows.len(), 10);
        assert_eq!(grid.cursor, 0);
        assert_eq!(grid.current_grid_row().buy_price, dec!(4.0));
        assert_eq!(grid.current_grid_row().sell_price, dec!(4.698));
        assert!(!grid.running.load(Ordering::Relaxed));
        assert!(!grid.locked.load(Ordering::Relaxed));

        let signal = grid.evaluate_with_price(&params, dec!(4.25));
        assert_eq!(signal, None);

        let signal = grid.evaluate_with_price(&params, dec!(4.0));
        assert_eq!(
            signal,
            Some(TradeSignal::Buy {
                price: dec!(4),
                quantity: dec!(25.0)
            })
        );
        assert_eq!(grid.locked.load(Ordering::Relaxed), true);

        let order = Order::builder()
            .exchange("Binace")
            .base_asset("DOT")
            .quote_asset("USDT")
            .symbol("DOTUSDT")
            .order_id("1")
            .price("4.0")
            .avg_price("4.0")
            .orig_qty("25.0")
            .executed_qty("25.0")
            .cumulative_quote_qty("25.0")
            .order_type(OrderType::Market)
            .order_side(OrderSide::Buy)
            .order_status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        grid.update_with_order(&signal.unwrap(), &order);

        assert_eq!(grid.current_grid_row().buyed, true);
        assert_eq!(grid.locked.load(Ordering::Relaxed), false);

        let signal = grid.evaluate_with_price(&params, dec!(3.99));
        assert_eq!(signal, None);

        let signal = grid.evaluate_with_price(&params, dec!(4.698));
        assert_eq!(
            signal,
            Some(TradeSignal::Sell {
                price: dec!(4.698),
                quantity: dec!(24.98)
            })
        );
        assert_eq!(grid.locked.load(Ordering::Relaxed), true);

        let order = Order::builder()
            .exchange("Binace")
            .base_asset("DOT")
            .quote_asset("USDT")
            .symbol("DOTUSDT")
            .order_id("2")
            .price("4.698")
            .avg_price("4.698")
            .orig_qty("24.98")
            .executed_qty("24.98")
            .cumulative_quote_qty("24.98")
            .order_type(OrderType::Market)
            .order_side(OrderSide::Sell)
            .order_status(OrderStatus::Filled)
            .time(0)
            .update_time(0)
            .build();

        grid.update_with_order(&signal.unwrap(), &order);

        assert_eq!(grid.current_grid_row().sold, true);
        assert_eq!(grid.locked.load(Ordering::Relaxed), false);

        let signal = grid.evaluate_with_price(&params, dec!(5.108));
        assert_eq!(signal, None);
        assert_eq!(grid.cursor, 0);

        let signal = grid.evaluate_with_price(&params, dec!(5.109));
        assert_eq!(signal, None);
        assert_eq!(grid.cursor, 1);

        let signal = grid.evaluate_with_price(&params, dec!(4.697));
        assert_eq!(signal, None);

        let signal = grid.evaluate_with_price(&params, dec!(6.001));
        assert_eq!(signal, None);
        assert_eq!(grid.cursor, 2);

        let signal = grid.evaluate_with_price(&params, dec!(5.519));
        assert_eq!(
            signal,
            Some(TradeSignal::Buy {
                price: dec!(5.519),
                quantity: dec!(18.12)
            })
        );

        Ok(())
    }
}

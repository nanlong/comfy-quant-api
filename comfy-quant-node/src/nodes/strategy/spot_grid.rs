use crate::{
    node_core::{Executable, Port, PortAccessor, Setupable, SpotTradeable},
    node_io::{SpotPairInfo, TickStream},
    workflow::{self, WorkflowContext},
};
use anyhow::{anyhow, Result};
use bon::{bon, Builder};
use comfy_quant_exchange::client::{
    spot_client::base::{Order, OrderSide},
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::str::FromStr;

#[derive(Builder, Debug, Clone)]
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

/// 网格交易
/// inputs:
///     0: SpotPairInfo
///     1: SpotClient
///     2: TickStream
///
/// outputs:
///     0: 持仓信息
///     1: 行情数据
///     2: 日志信息
///
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct SpotGrid {
    params: Params,                   // 前端配置
    port: Port,                       // 输入输出
    context: Option<WorkflowContext>, // 工作流上下文信息
    grid: Option<Grid>,               // 网格
    initialized: bool,                // 是否已经初始化
}

impl SpotGrid {
    pub(crate) fn new(params: Params) -> Result<Self> {
        Ok(SpotGrid {
            params,
            port: Port::new(),
            context: None,
            grid: None,
            initialized: false,
        })
    }

    pub(crate) async fn initialize(
        &mut self,
        pair_info: &SpotPairInfo,
        client: &SpotClientKind,
        current_price: Decimal,
    ) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        let balance = client.get_balance(&pair_info.quote_asset).await?;

        if balance.free.parse::<Decimal>()? < self.params.investment {
            anyhow::bail!("Insufficient free balance");
        }

        // 初始化策略可操作的账户金额
        self.get_context()?
            .assets()
            .add_value(&pair_info.quote_asset, self.params.investment);

        let _platform_name = client.platform_name();
        let symbol_info = client
            .get_symbol_info(&pair_info.base_asset, &pair_info.quote_asset)
            .await?;
        let account = client.get_account().await?;
        let base_asset_precision = symbol_info.base_asset_precision;
        let quote_asset_precision = symbol_info.quote_asset_precision;

        // 计算网格价格
        let grid_prices = calc_grid_prices(
            &self.params.mode,
            self.params.lower_price,
            self.params.upper_price,
            self.params.grid_rows,
            quote_asset_precision,
        );

        // 创建网格
        let grid = Grid::builder()
            .investment(self.params.investment)
            .grid_prices(grid_prices)
            .current_price(current_price)
            .base_asset_precision(base_asset_precision)
            .quote_asset_precision(quote_asset_precision)
            .commission_rate(account.taker_commission_rate)
            .build();

        self.grid = Some(grid);

        self.initialized = true;

        Ok(())
    }

    fn get_grid_mut(&mut self) -> Result<&mut Grid> {
        self.grid
            .as_mut()
            .ok_or_else(|| anyhow!("SpotGrid grid not initializer"))
    }
}

impl Setupable for SpotGrid {
    fn setup_context(&mut self, context: WorkflowContext) {
        self.context = Some(context);
    }

    fn get_context(&self) -> Result<&WorkflowContext> {
        self.context
            .as_ref()
            .ok_or_else(|| anyhow!("context not setup"))
    }
}

impl PortAccessor for SpotGrid {
    fn get_port(&self) -> Result<&Port> {
        Ok(&self.port)
    }

    fn get_port_mut(&mut self) -> Result<&mut Port> {
        Ok(&mut self.port)
    }
}

// 节点执行
impl Executable for SpotGrid {
    async fn execute(&mut self) -> Result<()> {
        // 等待其他节点
        self.get_context()?.wait().await?;

        // 获取输入
        let pair_info = self.port.get_input::<SpotPairInfo>(0)?;
        let client = self.port.get_input::<SpotClientKind>(1)?;
        let tick_stream = self.port.get_input::<TickStream>(2)?;

        let tick_rx = tick_stream.subscribe();
        let current_price = tick_rx.recv_async().await?.price;
        self.initialize(&pair_info, &client, current_price).await?;

        let params = self.params.clone();

        while let Ok(tick) = tick_rx.recv_async().await {
            let signal = self
                .get_grid_mut()?
                .evaluate_with_price(&params, tick.price);

            if let Some(signal) = signal {
                match signal {
                    TradeSignal::Buy(qty) => {
                        let order = self
                            .market_buy(
                                &client,
                                &pair_info.base_asset,
                                &pair_info.quote_asset,
                                qty.to_string().parse::<f64>()?,
                            )
                            .await?;

                        self.get_grid_mut()?.update_with_order(&signal, &order);
                        tracing::info!("SpotGrid buy order: {:?}", order);
                    }

                    TradeSignal::Sell(qty) => {
                        let order = self
                            .market_sell(
                                &client,
                                &pair_info.base_asset,
                                &pair_info.quote_asset,
                                qty.to_string().parse::<f64>()?,
                            )
                            .await?;
                        self.get_grid_mut()?.update_with_order(&signal, &order);
                        tracing::info!("SpotGrid sell order: {:?}", order);
                    }

                    TradeSignal::StopLoss(sell_all) => {
                        if !sell_all {
                            continue;
                        }

                        let balance = client.get_balance(&pair_info.base_asset).await?;
                        let order = self
                            .market_sell(
                                &client,
                                &pair_info.base_asset,
                                &pair_info.quote_asset,
                                balance.free.parse::<f64>()?,
                            )
                            .await?;
                        self.get_grid_mut()?.update_with_order(&signal, &order);
                        tracing::info!("SpotGrid sell all order: {:?}", order);
                    }

                    TradeSignal::TakeProfit => {
                        let balance = client.get_balance(&pair_info.base_asset).await?;

                        let order = self
                            .market_sell(
                                &client,
                                &pair_info.base_asset,
                                &pair_info.quote_asset,
                                balance.free.parse::<f64>()?,
                            )
                            .await?;
                        self.get_grid_mut()?.update_with_order(&signal, &order);
                        tracing::info!("SpotGrid take profit order: {:?}", order);
                    }
                }
            }
        }

        Ok(())
    }
}

impl TryFrom<&workflow::Node> for SpotGrid {
    type Error = anyhow::Error;

    fn try_from(node: &workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "strategy.SpotGrid" {
            anyhow::bail!("Try from workflow::Node to SpotGrid failed: Invalid prop_type");
        }

        let [mode, lower_price, upper_price, grid_rows, investment, trigger_price, stop_loss, take_profit, sell_all_on_stop] =
            node.properties.params.as_slice()
        else {
            anyhow::bail!("Try from workflow::Node to BinanceSubAccount failed: Invalid params");
        };

        let mode = mode
            .as_str()
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to SpotGrid failed: Invalid mode"
            ))?
            .parse::<Mode>()?;

        let lower_price = lower_price
            .as_f64()
            .and_then(|p| Decimal::try_from(p).ok())
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to SpotGrid failed: Invalid lower_price"
            ))?;

        let upper_price = upper_price
            .as_f64()
            .and_then(|p| Decimal::try_from(p).ok())
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to SpotGrid failed: Invalid upper_price"
            ))?;

        if lower_price >= upper_price {
            anyhow::bail!(
                "Try from workflow::Node to SpotGrid failed: Invalid lower_price and upper_price"
            );
        }

        let grid_rows = grid_rows.as_u64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to SpotGrid failed: Invalid grid_rows"
        ))?;

        if !(2..150).contains(&grid_rows) {
            anyhow::bail!("Try from workflow::Node to SpotGrid failed: Invalid grid_rows");
        }

        let investment = investment
            .as_f64()
            .and_then(|p| Decimal::try_from(p).ok())
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to SpotGrid failed: Invalid investment"
            ))?;

        let trigger_price = trigger_price
            .as_f64()
            .and_then(|p| Decimal::try_from(p).ok());

        let stop_loss = stop_loss.as_f64().and_then(|p| Decimal::try_from(p).ok());

        let take_profit = take_profit.as_f64().and_then(|p| Decimal::try_from(p).ok());

        let sell_all_on_stop = sell_all_on_stop.as_bool().unwrap_or(true);

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

        SpotGrid::new(params)
    }
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug)]
pub(crate) struct Grid {
    rows: Vec<GridRow>,       // 网格行
    cursor: usize,            // 当前网格序号
    prev_sell_price: Decimal, // 上一次的卖出价格
    starting: bool,           // 是否开始
    running: bool,            // 是否运行
    locked: bool,             // 是否锁定
}

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub(crate) enum TradeSignal {
    Buy(Decimal),   // 买入数量
    Sell(Decimal),  // 卖出数量
    StopLoss(bool), // 止损
    TakeProfit,     // 止盈
}

#[allow(unused)]
#[bon]
impl Grid {
    #[builder]
    fn new(
        investment: Decimal,        // 投资金额
        grid_prices: Vec<Decimal>,  // 网格价格
        current_price: Decimal,     // 当前价格
        base_asset_precision: u32,  // 基础币种小数点位数
        quote_asset_precision: u32, // 报价币种小数点位数
        commission_rate: Decimal,   // 手续费
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

        let starting = true;
        let running = false;
        let locked = false;

        Grid {
            rows,
            cursor,
            prev_sell_price,
            starting,
            running,
            locked,
        }
    }

    /// 根据当前价格，获取交易信号
    fn evaluate_with_price(
        &mut self,
        params: &Params,
        current_price: Decimal,
    ) -> Option<TradeSignal> {
        let price_tolerance = dec!(0.005);

        // 如果未运行或锁定，则不进行操作
        if !self.starting || self.locked {
            return None;
        }

        if !self.running {
            // 如果设置了触发价格，则根据触发价格决定是否运行
            if let Some(trigger_price) = params.trigger_price {
                if current_price <= trigger_price {
                    self.running = true;
                }
            } else {
                self.running = true;
            }

            return None;
        }

        // 止损
        if let Some(stop_loss) = params.stop_loss {
            if current_price <= stop_loss {
                self.starting = false;
                return Some(TradeSignal::StopLoss(params.sell_all_on_stop));
            }
        }

        // 止盈
        if let Some(take_profit) = params.take_profit {
            if current_price >= take_profit {
                self.starting = false;
                return Some(TradeSignal::TakeProfit);
            }
        }

        // 当前格子
        let current_grid = self.current_grid();

        // 买入
        if !current_grid.buyed // 当前格子未买入
            && self.prev_sell_price != current_grid.buy_price // 上一次的卖出价格不等于当前的买入价格
            && current_price <= current_grid.buy_price
            && current_price > current_grid.buy_price * (dec!(1) - price_tolerance)
        {
            let buy_quantity = current_grid.buy_quantity;
            self.locked = true;
            return Some(TradeSignal::Buy(buy_quantity));
        }

        // 卖出
        if current_grid.buyed // 当前格子已买入
            && !current_grid.sold // 当前格子未卖出
            && current_price >= current_grid.sell_price
            && current_price < current_grid.sell_price * (dec!(1) + price_tolerance)
        {
            let sell_quantity = current_grid.sell_quantity;
            let sell_price = current_grid.sell_price;
            self.locked = true;
            self.prev_sell_price = sell_price;
            return Some(TradeSignal::Sell(sell_quantity));
        }

        // 向下移动一格
        if current_price < current_grid.buy_price {
            if let Some(lower_grid) = self.lower_grid() {
                let step = (lower_grid.sell_price - lower_grid.buy_price) / dec!(2);
                if current_price <= current_grid.buy_price - step {
                    self.cursor -= 1;
                    return None;
                }
            }
        }

        // 向上移动一格
        if current_price > current_grid.sell_price {
            if let Some(upper_grid) = self.upper_grid() {
                let step = (upper_grid.sell_price - upper_grid.buy_price) / dec!(2);
                if current_price >= current_grid.sell_price + step {
                    self.cursor += 1;
                    return None;
                }
            }
        }

        None
    }

    /// 更新网格状态
    fn update_with_order(&mut self, signal: &TradeSignal, order: &Order) {
        let current_grid = self.current_grid_mut();

        match order.order_side {
            OrderSide::Buy => {
                current_grid.buyed = true;
                current_grid.sold = false;
            }
            OrderSide::Sell => {
                current_grid.sold = true;
                current_grid.buyed = false;
            }
        }

        self.locked = false;
    }

    /// 获取当前行
    fn current_grid(&self) -> &GridRow {
        &self.rows[self.cursor]
    }

    /// 获取当前行可变引用
    fn current_grid_mut(&mut self) -> &mut GridRow {
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

#[derive(Builder, Debug)]
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
    use comfy_quant_exchange::client::spot_client::base::{OrderStatus, OrderType};

    #[test]
    fn test_try_from_node_to_spot_grid() -> Result<()> {
        let json_str = r#"{"id":4,"type":"交易策略/网格(现货)","pos":[367,125],"size":{"0":210,"1":310},"flags":{},"order":1,"mode":0,"inputs":[{"name":"交易所信息","type":"exchangeData","link":null},{"name":"最新成交价格","type":"tickerStream","link":null},{"name":"账户","type":"account","link":null},{"name":"回测","type":"backtest","link":null}],"properties":{"type":"strategy.SpotGrid","params":["arithmetic",1,1.1,8,1,"","","",true]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;

        let spot_grid = SpotGrid::try_from(&node)?;

        assert_eq!(spot_grid.params.mode, Mode::Arithmetic);
        assert_eq!(spot_grid.params.lower_price, Decimal::try_from(1.0)?);
        assert_eq!(spot_grid.params.upper_price, Decimal::try_from(1.1)?);
        assert_eq!(spot_grid.params.grid_rows, 8);
        assert_eq!(spot_grid.params.investment, Decimal::try_from(1.0)?);
        assert_eq!(spot_grid.params.trigger_price, None);
        assert_eq!(spot_grid.params.stop_loss, None);
        assert_eq!(spot_grid.params.take_profit, None);
        assert_eq!(spot_grid.params.sell_all_on_stop, true);

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
            .investment(params.investment)
            .grid_prices(grid_prices)
            .base_asset_precision(base_asset_precision)
            .quote_asset_precision(quote_asset_precision)
            .current_price(dec!(4.25))
            .commission_rate(commission)
            .build();

        assert_eq!(grid.rows.len(), 10);
        assert_eq!(grid.cursor, 0);
        assert_eq!(grid.current_grid().buy_price, dec!(4.0));
        assert_eq!(grid.current_grid().sell_price, dec!(4.698));
        assert!(!grid.running);
        assert!(!grid.locked);

        let signal = grid.evaluate_with_price(&params, dec!(4.25));
        assert_eq!(signal, None);

        let signal = grid.evaluate_with_price(&params, dec!(4.0));
        assert_eq!(signal, Some(TradeSignal::Buy(dec!(25.0))));
        assert_eq!(grid.locked, true);

        let order = Order::builder()
            .symbol("DOTUSDT")
            .order_id("1")
            .price("4.0")
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

        assert_eq!(grid.current_grid().buyed, true);
        assert_eq!(grid.locked, false);

        let signal = grid.evaluate_with_price(&params, dec!(3.99));
        assert_eq!(signal, None);

        let signal = grid.evaluate_with_price(&params, dec!(4.698));
        assert_eq!(signal, Some(TradeSignal::Sell(dec!(24.98))));
        assert_eq!(grid.locked, true);

        let order = Order::builder()
            .symbol("DOTUSDT")
            .order_id("2")
            .price("4.698")
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

        assert_eq!(grid.current_grid().sold, true);
        assert_eq!(grid.locked, false);

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
        assert_eq!(signal, Some(TradeSignal::Buy(dec!(18.12))));

        Ok(())
    }
}

use crate::{
    node_core::{Executable, PortManager, Ports},
    node_io::{SpotPairInfo, TickStream},
    workflow,
};
use anyhow::Result;
use bon::Builder;
use comfy_quant_exchange::client::spot_client_kind::{SpotClientKind, SpotExchangeClient};
use std::str::FromStr;

#[allow(unused)]
#[derive(Builder, Debug, Clone)]
pub struct Widget {
    mode: Mode,                 // 网格模式
    lower_price: f64,           // 网格下界
    upper_price: f64,           // 网格上界
    grids: u64,                 // 网格数量
    investment: f64,            // 投资金额
    trigger_price: Option<f64>, // 触发价格
    stop_loss: Option<f64>,     // 止损价格
    take_profit: Option<f64>,   // 止盈价格
    sell_all_on_stop: bool,     // 是否在止损时卖出所有基准币，默认为true
}

#[derive(Debug)]
#[allow(unused)]
pub struct SpotGrid {
    pub(crate) widget: Widget,
    // inputs:
    //      0: SpotPairInfo
    //      1: SpotClient
    //      2: TickStream
    pub(crate) ports: Ports,
    // 要交易的币种信息
    // pub(crate) pair: Option<Pair>,

    // 网格价格
    // pub(crate) grids: Vec<f64>,

    // 账户信息, 总余额、
    // pub(crate) account: Option<Account>,

    // 持仓信息
    // pub(crate) position: {base_currency: f64, quote_currency: f64},

    // 订单信息，提交 - 确认，流程完成后删除
    // pub(crate) orders: Vec<Order>,
}

impl SpotGrid {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let ports = Ports::new();

        Ok(SpotGrid { widget, ports })
    }
}

impl PortManager for SpotGrid {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

// 节点执行
#[allow(unused)]
impl Executable for SpotGrid {
    async fn execute(&mut self) -> Result<()> {
        let slot0 = self.ports.get_input::<SpotPairInfo>(0)?;
        let slot1 = self.ports.get_input::<SpotClientKind>(1)?;
        let slot2 = self.ports.get_input::<TickStream>(2)?;

        let pair_info = slot0.inner();
        let client = slot1.inner();

        tokio::spawn(async move {
            let rx = slot2.subscribe();

            while let Ok(tick) = rx.recv_async().await {
                dbg!(&tick);
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        dbg!(&pair_info);
        dbg!(&client);

        dbg!(client.get_account().await?);

        let decimals = 10;
        let maker_commission = 0.001;
        let taker_commission = 0.001;

        // 根据最低价、最高价、网格数量计算网格价格
        let grids = calculate_grids(
            self.widget.mode.clone(),
            self.widget.lower_price,
            self.widget.upper_price,
            self.widget.grids,
            decimals,
        );

        // 计算每格利润率
        let profit = calculate_grid_profit(
            self.widget.mode.clone(),
            self.widget.lower_price,
            self.widget.upper_price,
            taker_commission,
            self.widget.grids,
        );

        // tokio::spawn(async move {
        //     let rx = slot.subscribe();

        //     while let Ok(tick) = rx.recv_async().await {
        //         dbg!(&tick);
        //     }

        //     println!("tick_stream.next().await is done");

        //     #[allow(unreachable_code)]
        //     Ok::<(), anyhow::Error>(())
        // });

        Ok(())
    }
}

impl TryFrom<workflow::Node> for SpotGrid {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "strategy.SpotGrid" {
            anyhow::bail!("Try from workflow::Node to SpotGrid failed: Invalid prop_type");
        }

        let [mode, lower_price, upper_price, grids, investment, trigger_price, stop_loss, take_profit, sell_all_on_stop] =
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

        let lower_price = lower_price.as_f64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to SpotGrid failed: Invalid lower_price"
        ))?;

        let upper_price = upper_price.as_f64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to SpotGrid failed: Invalid upper_price"
        ))?;

        if lower_price >= upper_price {
            anyhow::bail!(
                "Try from workflow::Node to SpotGrid failed: Invalid lower_price and upper_price"
            );
        }

        let grids = grids.as_u64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to SpotGrid failed: Invalid grids"
        ))?;

        if !(2..150).contains(&grids) {
            anyhow::bail!("Try from workflow::Node to SpotGrid failed: Invalid grids");
        }

        let investment = investment.as_f64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to SpotGrid failed: Invalid investment"
        ))?;

        let trigger_price = trigger_price.as_f64();

        let stop_loss = stop_loss.as_f64();

        let take_profit = take_profit.as_f64();

        let sell_all_on_stop = sell_all_on_stop.as_bool().unwrap_or(true);

        let widget = Widget::builder()
            .mode(mode)
            .lower_price(lower_price)
            .upper_price(upper_price)
            .grids(grids)
            .investment(investment)
            .maybe_trigger_price(trigger_price)
            .maybe_stop_loss(stop_loss)
            .maybe_take_profit(take_profit)
            .sell_all_on_stop(sell_all_on_stop)
            .build();

        SpotGrid::try_new(widget)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
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
// 计算网格价格
fn calculate_grids(
    mode: Mode,       // 网格模式
    lower_price: f64, // 网格下界
    upper_price: f64, // 网格上界
    grids: u64,       // 网格数量
    decimals: u32,    // 小数点位数
) -> Vec<f64> {
    match mode {
        Mode::Arithmetic => {
            let step = (upper_price - lower_price) / grids as f64;
            (0..=grids)
                .map(|i| round_to(lower_price + step * i as f64, decimals))
                .collect()
        }
        Mode::Geometric => {
            let step = (upper_price / lower_price).powf(1.0 / grids as f64);
            (0..=grids)
                .map(|i| round_to(lower_price * step.powi(i as i32), decimals))
                .collect()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum GridProfitRate {
    Arithmetic { min_rate: f64, max_rate: f64 },
    Geometric { rate: f64 },
}

// 计算网格的每格利润率
// 参考资料：https://www.binance.com/zh-CN/support/faq/币安现货网格交易的参数说明-688ff6ff08734848915de76a07b953dd
fn calculate_grid_profit(
    mode: Mode,            // 网格模式
    lower_price: f64,      // 网格下界
    upper_price: f64,      // 网格上界
    taker_commission: f64, // 手续费
    grids: u64,            // 网格数量
) -> GridProfitRate {
    match mode {
        Mode::Arithmetic => {
            let step = (upper_price - lower_price) / grids as f64;
            let max_profit_rate =
                (1. - taker_commission) * step / lower_price - 2. * taker_commission;
            let min_profit_rate = (upper_price * (1. - taker_commission)) / (upper_price - step)
                - 1.
                - taker_commission;

            GridProfitRate::Arithmetic {
                min_rate: floor_to(min_profit_rate, 4),
                max_rate: floor_to(max_profit_rate, 4),
            }
        }
        Mode::Geometric => {
            let step = (upper_price / lower_price).powf(1.0 / grids as f64);
            let profit_rate = (1. - taker_commission) * step - 1. - taker_commission;

            GridProfitRate::Geometric {
                rate: floor_to(profit_rate, 4),
            }
        }
    }
}

#[allow(unused)]
fn calculate_minimum_investment(
    min_qty: f64,              // 最小交易数量
    min_notional: Option<f64>, // 最小名义价值
    upper_price: f64,          // 网格上界
    lower_price: f64,          // 网格下界
    grid_count: u32,           // 网格数量
    current_price: f64,        // 当前价格
) -> f64 {
    todo!()
}

// 保留小数点位数，向下取整
fn floor_to(f: f64, decimals: u32) -> f64 {
    let scale = 10_u64.pow(decimals);
    (f * scale as f64).floor() / scale as f64
}

// 保留小数点位数，四舍五入
fn round_to(f: f64, decimals: u32) -> f64 {
    let scale = 10_u64.pow(decimals);
    (f * scale as f64).round() / scale as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_spot_grid() -> Result<()> {
        let json_str = r#"{"id":4,"type":"交易策略/网格(现货)","pos":[367,125],"size":{"0":210,"1":310},"flags":{},"order":1,"mode":0,"inputs":[{"name":"交易所信息","type":"exchangeData","link":null},{"name":"最新成交价格","type":"tickerStream","link":null},{"name":"账户","type":"account","link":null},{"name":"回测","type":"backtest","link":null}],"properties":{"type":"strategy.SpotGrid","params":["arithmetic",1,1.1,8,1,"","","",true]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;

        let spot_grid = SpotGrid::try_from(node)?;

        assert_eq!(spot_grid.widget.mode, Mode::Arithmetic);
        assert_eq!(spot_grid.widget.lower_price, 1.0);
        assert_eq!(spot_grid.widget.upper_price, 1.1);
        assert_eq!(spot_grid.widget.grids, 8);
        assert_eq!(spot_grid.widget.investment, 1.0);
        assert_eq!(spot_grid.widget.trigger_price, None);
        assert_eq!(spot_grid.widget.stop_loss, None);
        assert_eq!(spot_grid.widget.take_profit, None);
        assert_eq!(spot_grid.widget.sell_all_on_stop, true);

        Ok(())
    }

    #[test]
    fn test_calculate_grids() -> Result<()> {
        let grids = calculate_grids(Mode::Arithmetic, 1.0, 1.1, 8, 3);
        assert_eq!(
            grids,
            vec![1.0, 1.013, 1.025, 1.038, 1.05, 1.063, 1.075, 1.088, 1.1]
        );

        let grids = calculate_grids(Mode::Geometric, 1.0, 1.1, 8, 3);
        assert_eq!(
            grids,
            vec![1.0, 1.012, 1.024, 1.036, 1.049, 1.061, 1.074, 1.087, 1.1]
        );

        let grids = calculate_grids(Mode::Geometric, 4.0, 20.0, 10, 3);
        assert_eq!(
            grids,
            vec![4.0, 4.698, 5.519, 6.483, 7.615, 8.944, 10.506, 12.341, 14.496, 17.027, 20.0]
        );

        let grids = calculate_grids(Mode::Geometric, 4.0, 20.0, 2, 3);
        assert_eq!(grids, vec![4.0, 8.944, 20.0]);

        Ok(())
    }

    #[test]
    fn test_calculate_grid_profit() -> Result<()> {
        let profit = calculate_grid_profit(Mode::Arithmetic, 4.0, 20.0, 0.001, 10);
        assert_eq!(
            profit,
            GridProfitRate::Arithmetic {
                min_rate: 0.0848,
                max_rate: 0.3976
            }
        );

        let profit = calculate_grid_profit(Mode::Geometric, 4.0, 20.0, 0.001, 10);
        assert_eq!(profit, GridProfitRate::Geometric { rate: 0.1724 });

        Ok(())
    }
}

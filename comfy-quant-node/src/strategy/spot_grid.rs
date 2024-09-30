/// 网格交易策略
/// 1. 订单系统
use crate::{
    traits::{NodeDataPort, NodeExecutor},
    workflow, DataPorts,
};
use anyhow::Result;
use std::str::FromStr;
use tokio::sync::broadcast;

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
        match s {
            "arithmetic" => Ok(Mode::Arithmetic),
            "geometric" => Ok(Mode::Geometric),
            _ => Err(anyhow::anyhow!("Invalid mode: {}", s)),
        }
    }
}

pub struct Widget {
    // 网格模式
    mode: Mode,
    // 网格下界
    lower_price: f64,
    // 网格上界
    upper_price: f64,
    // 网格数量
    grids: u64,
    // 投资金额
    investment: f64,
    // 触发价格
    trigger_price: Option<f64>,
    // 止损价格
    stop_loss: Option<f64>,
    // 止盈价格
    take_profit: Option<f64>,
    // 是否在止损时卖出所有基准币，默认为true
    sell_all_on_stop: bool,
}

impl Widget {
    pub fn new(
        mode: Mode,
        lower_price: f64,
        upper_price: f64,
        grids: u64,
        investment: f64,
        trigger_price: Option<f64>,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
        sell_all_on_stop: bool,
    ) -> Self {
        Self {
            mode,
            lower_price,
            upper_price,
            grids,
            investment,
            trigger_price,
            stop_loss,
            take_profit,
            sell_all_on_stop,
        }
    }
}

pub struct SpotGrid {
    pub(crate) widget: Widget,
    pub(crate) data_ports: DataPorts,
}

impl SpotGrid {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let data_ports = DataPorts::new(5, 0);
        Ok(SpotGrid { widget, data_ports })
    }
}

impl NodeDataPort for SpotGrid {
    fn get_data_port(&self) -> Result<&DataPorts> {
        Ok(&self.data_ports)
    }

    fn get_data_port_mut(&mut self) -> Result<&mut DataPorts> {
        Ok(&mut self.data_ports)
    }
}

impl NodeExecutor for SpotGrid {
    async fn execute(&mut self) -> Result<()> {
        // 根据最低价、最高价、网格数量计算网格价格
        let grid_prices = calculate_grids(
            self.widget.mode.clone(),
            self.widget.lower_price,
            self.widget.upper_price,
            self.widget.grids,
            10,
        );

        // 根据总投资额、网格数量、每格收益率计算每格投资额
        let grid_investment = self.widget.investment / self.widget.grids as f64;

        Ok(())
    }
}

impl TryFrom<workflow::Node> for SpotGrid {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "strategy.spotGrid" {
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

        let widget = Widget::new(
            mode,
            lower_price,
            upper_price,
            grids,
            investment,
            trigger_price,
            stop_loss,
            take_profit,
            sell_all_on_stop,
        );

        SpotGrid::try_new(widget)
    }
}

// 计算网格价格
fn calculate_grids(
    mode: Mode,
    lower_price: f64,
    upper_price: f64,
    grids: u64,
    decimals: u32,
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

struct GridProfit {
    // 最小收益率
    min_rate: f64,
    // 最大收益率
    max_rate: f64,
}

// 计算网格利润
fn calculate_grid_profit(
    mode: Mode,
    lower_price: f64,
    upper_price: f64,
    grids: u64,
    investment: f64,
) -> GridProfit {
    let grid_prices = calculate_grids(mode, lower_price, upper_price, grids, 8);
    let mut min_rate = f64::MAX;
    let mut max_rate = f64::MIN;

    for i in 0..grid_prices.len() - 1 {
        let buy_price = grid_prices[i];
        let sell_price = grid_prices[i + 1];
        let profit_rate = (sell_price - buy_price) / buy_price;

        min_rate = min_rate.min(profit_rate);
        max_rate = max_rate.max(profit_rate);
    }

    GridProfit { min_rate, max_rate }
}

// 保留小数点位数
fn round_to(price: f64, decimals: u32) -> f64 {
    let scale = 10_u64.pow(decimals);
    (price * scale as f64).round() / scale as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_spot_grid() -> Result<()> {
        let json_str = r#"{"id":4,"type":"交易策略/网格(现货)","pos":[367,125],"size":{"0":210,"1":310},"flags":{},"order":1,"mode":0,"inputs":[{"name":"交易所信息","type":"exchangeData","link":null},{"name":"最新成交价格","type":"tickerStream","link":null},{"name":"账户","type":"account","link":null},{"name":"回测","type":"backtest","link":null}],"properties":{"type":"strategy.spotGrid","params":["arithmetic",1,1.1,8,1,"","","",true]}}"#;

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
        let prices = calculate_grids(Mode::Arithmetic, 1.0, 1.1, 8, 6);
        assert_eq!(
            prices,
            vec![1.0, 1.0125, 1.025, 1.0375, 1.05, 1.0625, 1.075, 1.0875, 1.1]
        );

        let prices = calculate_grids(Mode::Geometric, 1.0, 1.1, 8, 6);
        assert_eq!(
            prices,
            vec![1.0, 1.011985, 1.024114, 1.036388, 1.048809, 1.061379, 1.074099, 1.086973, 1.1]
        );

        Ok(())
    }
}

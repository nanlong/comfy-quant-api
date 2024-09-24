use crate::{
    data::BacktestConfig,
    traits::{NodeDataPort, NodeExecutor},
    workflow, DataPorts,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

pub struct Widget {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl Widget {
    pub fn new(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        Widget {
            start_time,
            end_time,
        }
    }
}

pub struct Backtest {
    pub(crate) widget: Widget,
    pub(crate) data_ports: DataPorts,
}

impl Backtest {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut data_ports = DataPorts::new(0, 1);
        data_ports.add_output(0, broadcast::channel::<BacktestConfig>(1).0)?;
        Ok(Backtest { widget, data_ports })
    }

    async fn output0(&self) -> Result<()> {
        let tx = self.data_ports.get_output::<BacktestConfig>(0)?.clone();

        let backtest = BacktestConfig {
            start_time: self.widget.start_time.clone(),
            end_time: self.widget.end_time.clone(),
        };

        tokio::spawn(async move {
            while tx.receiver_count() > 0 {
                tx.send(backtest)?;
                break;
            }

            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

impl NodeDataPort for Backtest {
    fn get_data_port(&self) -> Result<&DataPorts> {
        Ok(&self.data_ports)
    }

    fn get_data_port_mut(&mut self) -> Result<&mut DataPorts> {
        Ok(&mut self.data_ports)
    }
}

impl NodeExecutor for Backtest {
    async fn execute(&mut self) -> Result<()> {
        self.output0().await?;
        Ok(())
    }
}

impl TryFrom<workflow::Node> for Backtest {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "utils.backtest" {
            anyhow::bail!("Try from workflow::Node to Backtest failed: Invalid prop_type");
        }

        let [start_time, end_time] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to Backtest failed: Invalid params");
        };

        let start_time = start_time.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to Backtest failed: Invalid api_secret, the example: 2024-01-01"
        ))?;

        let end_time = end_time.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to Backtest failed: Invalid secret, the example: 2024-01-01"
        ))?;

        let start_time = format!("{}T00:00:00Z", start_time)
            .parse::<DateTime<Utc>>()
            .map_err(|e| anyhow::anyhow!("Parse start_time failed: {} {}", start_time, e))?;

        let end_time = format!("{}T23:59:59Z", end_time)
            .parse::<DateTime<Utc>>()
            .map_err(|e| anyhow::anyhow!("Parse end_time failed: {} {}", end_time, e))?;

        let widget = Widget::new(start_time, end_time);
        Backtest::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_backtest() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"工具/回测","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"utils.backtest","params":["2024-01-01","2024-01-02"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let backtest = Backtest::try_from(node)?;

        let start_time = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>()?;
        let end_time = "2024-01-02T23:59:59Z".parse::<DateTime<Utc>>()?;

        assert_eq!(backtest.widget.start_time, start_time);
        assert_eq!(backtest.widget.end_time, end_time);

        Ok(())
    }
}

use crate::{
    node_core::{Executable, Port, PortAccessor, Setupable, Slot},
    workflow::{self, WorkflowContext},
};
use anyhow::{anyhow, Result};
use bon::Builder;
use comfy_quant_exchange::client::{
    spot_client::backtest_spot_client::BacktestSpotClient as Client,
    spot_client_kind::SpotClientKind,
};
use std::sync::Arc;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    assets: Vec<(String, f64)>, // 币种，余额
    commissions: f64,           // 手续费
}

// 模拟账户，用于交易系统回测时使用
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BacktestSpotClient {
    node: workflow::Node,
    params: Params,
    // outputs:
    //      0: SpotClient
    port: Port,
    context: Option<Arc<WorkflowContext>>,
}

impl BacktestSpotClient {
    pub(crate) fn try_new(node: workflow::Node, params: Params) -> Result<Self> {
        let mut port = Port::default();

        let client = Client::builder()
            .assets(&params.assets[..])
            .commissions(params.commissions)
            .build();

        let client_slot = Slot::<SpotClientKind>::new(client.into());

        port.add_output(0, client_slot)?;

        Ok(BacktestSpotClient {
            node,
            params,
            port,
            context: None,
        })
    }
}

impl Setupable for BacktestSpotClient {
    fn setup_context(&mut self, context: Arc<WorkflowContext>) {
        self.context = Some(context);
    }

    fn get_context(&self) -> Result<&Arc<WorkflowContext>> {
        self.context
            .as_ref()
            .ok_or_else(|| anyhow!("context not setup"))
    }
}

impl PortAccessor for BacktestSpotClient {
    fn get_port(&self) -> &Port {
        &self.port
    }

    fn get_port_mut(&mut self) -> &mut Port {
        &mut self.port
    }
}

impl Executable for BacktestSpotClient {
    async fn execute(&mut self) -> Result<()> {
        self.get_context()?.wait().await;

        Ok(())
    }
}

impl TryFrom<&workflow::Node> for BacktestSpotClient {
    type Error = anyhow::Error;

    fn try_from(node: &workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "client.BacktestSpotClient" {
            anyhow::bail!(
                "Try from workflow::Node to BacktestSpotClient failed: Invalid prop_type"
            );
        }

        let [commissions, assets] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to BacktestSpotClient failed: Invalid params");
        };

        let commissions = commissions.as_f64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BacktestSpotClient failed: Invalid commissions"
        ))?;

        let assets = assets
            .as_array()
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to BacktestSpotClient failed: Invalid assets"
            ))?
            .iter()
            .filter_map(|asset| {
                let asset_array = asset.as_array()?;
                let asset_name = asset_array.first()?.as_str()?.to_string();
                let asset_balance = asset_array.get(1)?.as_f64()?;
                Some((asset_name, asset_balance))
            })
            .collect::<Vec<(String, f64)>>();

        let params = Params::builder()
            .assets(assets)
            .commissions(commissions)
            .build();

        BacktestSpotClient::try_new(node.clone(), params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comfy_quant_exchange::client::spot_client_kind::SpotClientExecutable;
    use rust_decimal_macros::dec;

    #[test]
    fn test_try_from_node_to_mock_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = BacktestSpotClient::try_from(&node)?;

        assert_eq!(
            account.params.assets,
            vec![("BTC".to_string(), 10.0), ("USDT".to_string(), 10000.0)]
        );
        assert_eq!(account.params.commissions, 0.001);

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_account_execute() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = BacktestSpotClient::try_from(&node)?;
        let port = account.get_port();

        let client = port.get_output::<SpotClientKind>(0)?;

        let balance = client.get_balance("BTC").await?;
        assert_eq!(balance.free, "10");

        let balance = client.get_balance("USDT").await?;
        assert_eq!(balance.free, "10000");

        let account_information = client.get_account().await?;
        assert_eq!(account_information.maker_commission_rate, dec!(0.001));
        assert_eq!(account_information.taker_commission_rate, dec!(0.001));

        Ok(())
    }
}

use std::sync::Arc;

use crate::{
    node_core::{NodeContext, NodeExecutable, NodeInfo, NodeInfra, NodePort, Port, Slot},
    workflow::Node,
};
use anyhow::Result;
use bon::Builder;
use comfy_quant_exchange::client::{
    spot_client::backtest_spot_client::BacktestSpotClient as Client,
    spot_client_kind::SpotClientKind,
};

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    assets: Vec<(String, f64)>, // 币种，余额
    commissions: f64,           // 手续费
}

impl TryFrom<&Node> for Params {
    type Error = anyhow::Error;

    fn try_from(node: &Node) -> Result<Self> {
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

        Ok(params)
    }
}

// 模拟账户，用于交易系统回测时使用
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BacktestSpotClient {
    params: Params,
    // outputs:
    //      0: SpotClient
    infra: NodeInfra,
}

impl BacktestSpotClient {
    pub(crate) fn try_new(node: Node) -> Result<Self> {
        let params = Params::try_from(&node)?;
        let mut infra = NodeInfra::new(node);

        let client = Client::builder()
            .assets(&params.assets[..])
            .commissions(params.commissions)
            .build();

        let client_slot = Arc::new(Slot::<SpotClientKind>::new(client.into()));

        infra.port.set_output(0, client_slot)?;

        Ok(BacktestSpotClient { params, infra })
    }
}

impl NodePort for BacktestSpotClient {
    fn port(&self) -> &Port {
        &self.infra.port
    }

    fn port_mut(&mut self) -> &mut Port {
        &mut self.infra.port
    }
}

impl NodeInfo for BacktestSpotClient {
    fn node_context(&self) -> Result<NodeContext> {
        self.infra.node_context()
    }
}

impl NodeExecutable for BacktestSpotClient {
    async fn execute(&mut self) -> Result<()> {
        self.infra.workflow_context()?.wait().await;

        Ok(())
    }
}

impl TryFrom<Node> for BacktestSpotClient {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
        BacktestSpotClient::try_new(node)
    }
}

impl TryFrom<&BacktestSpotClient> for Node {
    type Error = anyhow::Error;

    fn try_from(backtest_spot_client: &BacktestSpotClient) -> Result<Self> {
        Ok(backtest_spot_client.infra.node.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::workflow::WorkflowContext;

    use super::*;
    use async_lock::Barrier;
    use comfy_quant_exchange::client::spot_client_kind::SpotClientExecutable;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;

    #[test]
    fn test_try_from_node_to_mock_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: Node = serde_json::from_str(json_str)?;
        let account = BacktestSpotClient::try_from(node)?;

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

        let node: Node = serde_json::from_str(json_str)?;
        let account = BacktestSpotClient::try_from(node)?;
        let port = account.port();

        let client = port.output::<SpotClientKind>(0)?;

        let balance = client.get_balance("BTC").await?;
        assert_eq!(balance.free, "10");

        let balance = client.get_balance("USDT").await?;
        assert_eq!(balance.free, "10000");

        let account_information = client.get_account().await?;
        assert_eq!(account_information.maker_commission_rate, dec!(0.001));
        assert_eq!(account_information.taker_commission_rate, dec!(0.001));

        Ok(())
    }

    #[test]
    fn test_invalid_prop_type() {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"invalid.type","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: Node = serde_json::from_str(json_str).unwrap();
        let result = BacktestSpotClient::try_from(node);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid prop_type"));
    }

    #[test]
    fn test_invalid_params_count() {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001]}}"#;

        let node: Node = serde_json::from_str(json_str).unwrap();
        let result = BacktestSpotClient::try_from(node);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid params"));
    }

    #[test]
    fn test_invalid_commissions_format() {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":["invalid", [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: Node = serde_json::from_str(json_str).unwrap();
        let result = BacktestSpotClient::try_from(node);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid commissions"));
    }

    #[test]
    fn test_invalid_assets_format() {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, "invalid"]}}"#;

        let node: Node = serde_json::from_str(json_str).unwrap();
        let result = BacktestSpotClient::try_from(node);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid assets"));
    }

    #[test]
    fn test_empty_assets() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, []]}}"#;

        let node: Node = serde_json::from_str(json_str)?;
        let account = BacktestSpotClient::try_from(node)?;

        assert!(account.params.assets.is_empty());
        Ok(())
    }

    #[sqlx::test]
    fn test_node_info(db: PgPool) -> anyhow::Result<()> {
        let json_str = r#"{"id":42,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10]]]}}"#;

        let mut node: Node = serde_json::from_str(json_str)?;
        let db = Arc::new(db);
        node.context = Some(Arc::new(WorkflowContext::new(db, Barrier::new(0))));

        let account = BacktestSpotClient::try_from(node)?;
        let ctx = account.node_context()?;

        assert_eq!(ctx.node_id, 42);
        assert_eq!(ctx.node_name, "client.BacktestSpotClient");

        Ok(())
    }

    #[test]
    fn test_port_methods() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10]]]}}"#;

        let node: Node = serde_json::from_str(json_str)?;
        let mut account = BacktestSpotClient::try_from(node)?;

        // Test port() returns the correct port
        let port = account.port();
        assert!(port.output::<SpotClientKind>(0).is_ok());

        // Test port_mut() returns mutable reference
        let port_mut = account.port_mut();
        assert!(port_mut.output::<SpotClientKind>(0).is_ok());

        Ok(())
    }
}

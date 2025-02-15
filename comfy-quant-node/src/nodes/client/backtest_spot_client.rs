use crate::{
    node_core::{NodeCore, NodeCoreExt, NodeExecutable, NodeInfra, Slot},
    workflow::Node,
};
use anyhow::Result;
use bon::Builder;
use comfy_quant_exchange::client::{
    spot_client::backtest_spot_client::BacktestSpotClient as Client,
    spot_client_kind::SpotClientKind,
};
use std::sync::Arc;

// 模拟账户，用于交易系统回测时使用
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BacktestSpotClient {
    params: Params,
    // outputs:
    //      0: SpotClient
    infra: NodeInfra,
}

impl NodeCore for BacktestSpotClient {
    fn node_infra(&self) -> &NodeInfra {
        &self.infra
    }

    fn node_infra_mut(&mut self) -> &mut NodeInfra {
        &mut self.infra
    }
}

impl BacktestSpotClient {
    pub(crate) fn try_new(node: Node) -> Result<Self> {
        let params = Params::try_from(&node)?;
        let infra = NodeInfra::new(node);

        Ok(BacktestSpotClient { params, infra })
    }
}

impl NodeExecutable for BacktestSpotClient {
    async fn setup(&mut self) -> Result<()> {
        let price_store = self.workflow_context()?.cloned_price_store();

        let client = Client::builder()
            .assets(&self.params.assets[..])
            .commissions(self.params.commissions)
            .price_store(price_store)
            .build();

        let client_slot = Arc::new(Slot::<SpotClientKind>::new(client.into()));

        self.port_mut().set_output(0, client_slot)?;

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

    fn try_from(value: &BacktestSpotClient) -> Result<Self> {
        Ok(value.node().clone())
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    assets: Vec<(String, f64)>, // 币种，余额
    commissions: f64,           // 手续费
}

impl TryFrom<&Node> for Params {
    type Error = BacktestSpotClientError;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        if node.properties.prop_type != "client.BacktestSpotClient" {
            return Err(BacktestSpotClientError::PropertyTypeMismatch);
        }

        let [commissions, assets] = node.properties.params.as_slice() else {
            return Err(BacktestSpotClientError::ParamsFormatError);
        };

        let commissions = commissions
            .as_f64()
            .ok_or(BacktestSpotClientError::CommissionsError)?;

        let assets = assets
            .as_array()
            .ok_or(BacktestSpotClientError::AssetsError)?
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

#[derive(thiserror::Error, Debug)]
pub enum BacktestSpotClientError {
    #[error("Invalid property type, expected 'client.BacktestSpotClient'")]
    PropertyTypeMismatch,

    #[error("Invalid parameters format")]
    ParamsFormatError,

    #[error("Invalid assets")]
    AssetsError,

    #[error("Invalid commissions")]
    CommissionsError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        node_core::{ExchangeRateManager, NodeCoreExt},
        workflow::{QuoteAsset, WorkflowContext},
    };
    use async_lock::RwLock;
    use comfy_quant_exchange::client::spot_client_kind::SpotClientExecutable;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;

    fn default_context(db: PgPool) -> Arc<WorkflowContext> {
        Arc::new(WorkflowContext::new(
            Arc::new(db),
            Arc::new(RwLock::new(QuoteAsset::new())),
            Arc::new(RwLock::new(ExchangeRateManager::default())),
            Arc::new(RwLock::new(0)),
        ))
    }

    #[sqlx::test]
    fn test_try_from_node_to_mock_account(db: PgPool) -> Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let mut node: Node = serde_json::from_str(json_str)?;
        node.context = Some(default_context(db));

        let account = BacktestSpotClient::try_from(node)?;

        assert_eq!(
            account.params.assets,
            vec![("BTC".to_string(), 10.0), ("USDT".to_string(), 10000.0)]
        );
        assert_eq!(account.params.commissions, 0.001);

        Ok(())
    }

    #[sqlx::test]
    async fn test_mock_account_execute(db: PgPool) -> Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let mut node: Node = serde_json::from_str(json_str)?;
        node.context = Some(default_context(db));

        let mut account = BacktestSpotClient::try_from(node)?;
        account.setup().await?;

        let client = account.port().output::<SpotClientKind>(0)?;

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
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid property type, expected 'client.BacktestSpotClient'"
        );
    }

    #[test]
    fn test_invalid_params_count() {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001]}}"#;

        let node: Node = serde_json::from_str(json_str).unwrap();
        let result = BacktestSpotClient::try_from(node);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid parameters format");
    }

    #[test]
    fn test_invalid_commissions_format() {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":["invalid", [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: Node = serde_json::from_str(json_str).unwrap();
        let result = BacktestSpotClient::try_from(node);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid commissions");
    }

    #[test]
    fn test_invalid_assets_format() {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, "invalid"]}}"#;

        let node: Node = serde_json::from_str(json_str).unwrap();
        let result = BacktestSpotClient::try_from(node);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid assets");
    }

    #[sqlx::test]
    fn test_empty_assets(db: PgPool) -> Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, []]}}"#;

        let mut node: Node = serde_json::from_str(json_str)?;
        node.context = Some(default_context(db));

        let account = BacktestSpotClient::try_from(node)?;

        assert!(account.params.assets.is_empty());
        Ok(())
    }

    #[sqlx::test]
    fn test_node_info(db: PgPool) -> Result<()> {
        let json_str = r#"{"id":42,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10]]]}}"#;

        let mut node: Node = serde_json::from_str(json_str)?;
        node.context = Some(default_context(db));

        let account = BacktestSpotClient::try_from(node)?;
        let ctx = account.node_context()?;

        assert_eq!(ctx.node_id(), 42);
        assert_eq!(ctx.node_name(), "client.BacktestSpotClient");

        Ok(())
    }

    #[sqlx::test]
    fn test_port_methods(db: PgPool) -> Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["BTC", 10]]]}}"#;

        let mut node: Node = serde_json::from_str(json_str)?;
        node.context = Some(default_context(db));

        let mut account = BacktestSpotClient::try_from(node)?;
        account.setup().await?;

        // Test port() returns the correct port
        let port = account.port();
        assert!(port.output::<SpotClientKind>(0).is_ok());

        // Test port_mut() returns mutable reference
        let port_mut = account.port_mut();
        assert!(port_mut.output::<SpotClientKind>(0).is_ok());

        Ok(())
    }
}

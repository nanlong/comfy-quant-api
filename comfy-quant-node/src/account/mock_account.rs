use super::ClientInformation;
use crate::{
    base::{
        traits::node::{NodeExecutor, NodePorts},
        Ports, Slot,
    },
    workflow,
};
use anyhow::Result;
use bon::Builder;
use serde_json::json;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Widget {
    assets: Vec<(String, f64)>, // 币种，余额
    commissions: f64,           // 手续费
    market: String,
}

// 模拟账户，用于交易系统回测时使用
#[allow(unused)]
pub struct MockAccount {
    pub(crate) widget: Widget,
    pub(crate) ports: Ports,
}

impl MockAccount {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut ports = Ports::new();

        let client_information = ClientInformation::builder()
            .client_type("mock_client")
            .data(json!({"assets": &widget.assets, "commissions": widget.commissions, "market": &widget.market}))
            .build();

        ports.add_output(
            0,
            Slot::<ClientInformation>::builder()
                .data(client_information)
                .build(),
        )?;

        Ok(MockAccount { widget, ports })
    }
}

impl NodePorts for MockAccount {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for MockAccount {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<workflow::Node> for MockAccount {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "account.mockAccount" {
            anyhow::bail!("Try from workflow::Node to MockAccount failed: Invalid prop_type");
        }

        let [assets, commissions, market] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to MockAccount failed: Invalid params");
        };

        let assets = assets
            .as_array()
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to MockAccount failed: Invalid assets"
            ))?
            .into_iter()
            .filter_map(|asset| {
                let asset_array = asset.as_array()?;
                let asset_name = asset_array.get(0)?.as_str()?.to_string();
                let asset_balance = asset_array.get(1)?.as_f64()?;
                Some((asset_name, asset_balance))
            })
            .collect::<Vec<(String, f64)>>();

        let commissions = commissions.as_f64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to MockAccount failed: Invalid commissions"
        ))? / 100.0;

        let market = market.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to MockAccount failed: Invalid market"
        ))?;

        let widget = Widget::builder()
            .assets(assets)
            .commissions(commissions)
            .market(market)
            .build();

        MockAccount::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_mock_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.mockAccount","params":[[["BTC", 10], ["USDT", 10000]], 0.1, "spot"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = MockAccount::try_from(node)?;

        assert_eq!(
            account.widget.assets,
            vec![("BTC".to_string(), 10.0), ("USDT".to_string(), 10000.0)]
        );
        assert_eq!(account.widget.commissions, 0.001);
        assert_eq!(account.widget.market, "spot");

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_account_execute() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.mockAccount","params":[[["BTC", 10], ["USDT", 10000]], 0.1, "spot"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = MockAccount::try_from(node)?;

        let slot0 = account.ports.get_output::<ClientInformation>(0)?;

        let client_information = slot0.data().unwrap();

        assert_eq!(client_information.client_type, "mock_client");
        assert_eq!(
            client_information.data,
            Some(
                json!({"assets": [["BTC", 10.0], ["USDT", 10000.0]], "commissions": 0.001, "market": "spot"})
            )
        );

        Ok(())
    }
}

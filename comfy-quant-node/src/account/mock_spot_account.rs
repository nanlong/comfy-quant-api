use crate::{
    base::{
        traits::node::{NodeExecutor, NodePorts},
        Ports, Slot,
    },
    workflow,
};
use anyhow::Result;
use bon::Builder;
use comfy_quant_exchange::client::{MockSpotClient, SpotClient};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Widget {
    assets: Vec<(String, f64)>, // 币种，余额
    commissions: f64,           // 手续费
}

// 模拟账户，用于交易系统回测时使用
#[allow(unused)]
pub struct MockSpotAccount {
    pub(crate) widget: Widget,
    pub(crate) ports: Ports,
}

impl MockSpotAccount {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut ports = Ports::new();

        let client = MockSpotClient::builder()
            .assets(widget.assets.clone())
            .commissions(widget.commissions)
            .build();

        let output_slot0 = Slot::<Arc<Mutex<SpotClient>>>::builder()
            .data(Arc::new(Mutex::new(client.into())))
            .build();

        ports.add_output(0, output_slot0)?;

        Ok(MockSpotAccount { widget, ports })
    }
}

impl NodePorts for MockSpotAccount {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for MockSpotAccount {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<workflow::Node> for MockSpotAccount {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "account.mockSpotAccount" {
            anyhow::bail!("Try from workflow::Node to MockSpotAccount failed: Invalid prop_type");
        }

        let [assets, commissions] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to MockSpotAccount failed: Invalid params");
        };

        let assets = assets
            .as_array()
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to MockSpotAccount failed: Invalid assets"
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
            "Try from workflow::Node to MockSpotAccount failed: Invalid commissions"
        ))? / 100.0;

        let widget = Widget::builder()
            .assets(assets)
            .commissions(commissions)
            .build();

        MockSpotAccount::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use comfy_quant_exchange::client::SpotExchangeClient;

    use super::*;

    #[test]
    fn test_try_from_node_to_mock_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.mockSpotAccount","params":[[["BTC", 10], ["USDT", 10000]], 0.1]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = MockSpotAccount::try_from(node)?;

        assert_eq!(
            account.widget.assets,
            vec![("BTC".to_string(), 10.0), ("USDT".to_string(), 10000.0)]
        );
        assert_eq!(account.widget.commissions, 0.001);

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_account_execute() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.mockSpotAccount","params":[[["BTC", 10.0], ["USDT", 10000.0]], 0.1]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = MockSpotAccount::try_from(node)?;

        let slot0 = account.ports.get_output::<Arc<Mutex<SpotClient>>>(0)?;

        let client = slot0.data().unwrap();
        let client_guard = client.lock().await;

        let balance = client_guard.get_balance("BTC").await?;
        assert_eq!(balance.free, "10");

        let balance = client_guard.get_balance("USDT").await?;
        assert_eq!(balance.free, "10000");

        let account_information = client_guard.get_account().await?;
        assert_eq!(account_information.maker_commission, 0.001);
        assert_eq!(account_information.taker_commission, 0.001);

        Ok(())
    }
}

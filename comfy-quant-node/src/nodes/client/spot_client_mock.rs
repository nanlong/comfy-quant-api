use crate::{
    node_core::{Executable, PortAccessor, Ports, Slot},
    workflow,
};
use anyhow::Result;
use bon::Builder;
use comfy_quant_exchange::client::{
    spot_client::mock_spot_client::MockSpotClient, spot_client_kind::SpotClientKind,
};

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Widget {
    assets: Vec<(String, f64)>, // 币种，余额
    commissions: f64,           // 手续费
}

// 模拟账户，用于交易系统回测时使用
#[derive(Debug)]
#[allow(unused)]
pub struct SpotClientMock {
    pub(crate) widget: Widget,
    // outputs:
    //      0: SpotClient
    pub(crate) ports: Ports,
}

impl SpotClientMock {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut ports = Ports::new();

        let client = MockSpotClient::builder()
            .assets(widget.assets.clone())
            .commissions(widget.commissions)
            .build();

        let output_slot0 = Slot::<SpotClientKind>::builder()
            .data(client.into())
            .build();

        ports.add_output(0, output_slot0)?;

        Ok(SpotClientMock { widget, ports })
    }
}

impl PortAccessor for SpotClientMock {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl Executable for SpotClientMock {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<workflow::Node> for SpotClientMock {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "client.SpotClientMock" {
            anyhow::bail!("Try from workflow::Node to SpotClientMock failed: Invalid prop_type");
        }

        let [commissions, assets] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to SpotClientMock failed: Invalid params");
        };

        let commissions = commissions.as_f64().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to SpotClientMock failed: Invalid commissions"
        ))?;

        let assets = assets
            .as_array()
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to SpotClientMock failed: Invalid assets"
            ))?
            .into_iter()
            .filter_map(|asset| {
                let asset_array = asset.as_array()?;
                let asset_name = asset_array.get(0)?.as_str()?.to_string();
                let asset_balance = asset_array.get(1)?.as_f64()?;
                Some((asset_name, asset_balance))
            })
            .collect::<Vec<(String, f64)>>();

        let widget = Widget::builder()
            .assets(assets)
            .commissions(commissions)
            .build();

        SpotClientMock::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comfy_quant_exchange::client::spot_client_kind::SpotExchangeClient;

    #[test]
    fn test_try_from_node_to_mock_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.SpotClientMock","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = SpotClientMock::try_from(node)?;

        assert_eq!(
            account.widget.assets,
            vec![("BTC".to_string(), 10.0), ("USDT".to_string(), 10000.0)]
        );
        assert_eq!(account.widget.commissions, 0.001);

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_account_execute() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.SpotClientMock","params":[0.001, [["BTC", 10], ["USDT", 10000]]]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = SpotClientMock::try_from(node)?;

        let slot0 = account.ports.get_output::<SpotClientKind>(0)?;

        let client = slot0.inner();

        let balance = client.get_balance("BTC").await?;
        assert_eq!(balance.free, "10");

        let balance = client.get_balance("USDT").await?;
        assert_eq!(balance.free, "10000");

        let account_information = client.get_account().await?;
        assert_eq!(account_information.maker_commission, 0.001);
        assert_eq!(account_information.taker_commission, 0.001);

        Ok(())
    }
}

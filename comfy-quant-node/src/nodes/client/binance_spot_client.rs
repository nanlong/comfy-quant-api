use crate::{
    node_core::{
        traits::{NodeExecutor, NodePorts},
        Ports,
    },
    workflow,
};
use anyhow::Result;
use bon::Builder;
// use comfy_quant_exchange::client::SpotClient;
// use std::sync::Arc;
// use tokio::sync::Mutex;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
#[allow(unused)]
pub struct Widget {
    api_key: String,
    secret_key: String,
}

#[allow(unused)]
pub struct BinanceSpotClient {
    pub(crate) widget: Widget,
    // outputs:
    //      0: SpotClient
    pub(crate) ports: Ports,
}

impl BinanceSpotClient {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let ports = Ports::new();

        // todo: 创建SpotClient
        // let output_slot0 = Slot::<Arc<Mutex<SpotClient>>>::builder().build();

        // ports.add_output(0, output_slot0)?;

        Ok(BinanceSpotClient { widget, ports })
    }
}

impl NodePorts for BinanceSpotClient {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for BinanceSpotClient {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<workflow::Node> for BinanceSpotClient {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "account.binanceSubAccount" {
            anyhow::bail!("Try from workflow::Node to BinanceSpotClient failed: Invalid prop_type");
        }

        let [api_key, secret_key] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to BinanceSpotClient failed: Invalid params");
        };

        let api_key = api_key.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSpotClient failed: Invalid api_key"
        ))?;

        let secret_key = secret_key.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSpotClient failed: Invalid secret"
        ))?;

        let widget = Widget::builder()
            .api_key(api_key)
            .secret_key(secret_key)
            .build();

        BinanceSpotClient::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_binance_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = BinanceSpotClient::try_from(node)?;

        assert_eq!(account.widget.api_key, "api_secret");
        assert_eq!(account.widget.secret_key, "secret");

        Ok(())
    }

    // #[tokio::test]
    // async fn test_binance_account_execute() -> anyhow::Result<()> {
    //     let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret"]}}"#;

    //     let node: workflow::Node = serde_json::from_str(json_str)?;
    //     let account = BinanceSpotClient::try_from(node)?;

    //     // let _slot0 = account.ports.get_output::<Arc<Mutex<SpotClient>>>(0)?;

    //     // let client = slot0.data().unwrap();

    //     // assert_eq!(client_information.client_type, "binance_client");
    //     // assert_eq!(
    //     //     client_information.data,
    //     //     Some(json!({"api_key": "api_secret", "secret_key": "secret", "market": "spot"}))
    //     // );

    //     Ok(())
    // }
}

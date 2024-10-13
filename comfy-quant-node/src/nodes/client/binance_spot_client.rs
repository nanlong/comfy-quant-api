use crate::{
    node_core::{Executable, Port, PortAccessor},
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
pub(crate) struct Widget {
    api_key: String,
    secret_key: String,
}

#[allow(unused)]
pub(crate) struct BinanceSpotClient {
    pub(crate) widget: Widget,
    // outputs:
    //      0: SpotClient
    pub(crate) port: Port,
}

impl BinanceSpotClient {
    pub(crate) fn try_new(widget: Widget) -> Result<Self> {
        let port = Port::new();

        // todo: 创建SpotClient
        // let output_slot0 = Slot::<Arc<Mutex<SpotClient>>>::builder().build();

        // port.add_output(0, output_slot0)?;

        Ok(BinanceSpotClient { widget, port })
    }
}

impl PortAccessor for BinanceSpotClient {
    fn get_port(&self) -> Result<&Port> {
        Ok(&self.port)
    }

    fn get_port_mut(&mut self) -> Result<&mut Port> {
        Ok(&mut self.port)
    }
}

impl Executable for BinanceSpotClient {
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

    //     // let _slot0 = account.port.get_output::<Arc<Mutex<SpotClient>>>(0)?;

    //     // let client = slot0.data().unwrap();

    //     // assert_eq!(client_information.client_type, "binance_client");
    //     // assert_eq!(
    //     //     client_information.data,
    //     //     Some(json!({"api_key": "api_secret", "secret_key": "secret", "market": "spot"}))
    //     // );

    //     Ok(())
    // }
}

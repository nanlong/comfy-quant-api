use crate::{
    node_core::{NodeExecutable, NodeInfra, NodePort, Port, Slot},
    workflow::Node,
};
use anyhow::Result;
use bon::Builder;
use comfy_quant_exchange::client::{
    spot_client::binance_spot_client::BinanceSpotClient as Client, spot_client_kind::SpotClientKind,
};
use std::sync::Arc;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
#[allow(unused)]
pub(crate) struct Params {
    api_key: String,
    secret_key: String,
}

impl TryFrom<&Node> for Params {
    type Error = anyhow::Error;

    fn try_from(node: &Node) -> Result<Self> {
        if node.properties.prop_type != "client.BinanceSpotClient" {
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

        let params = Params::builder()
            .api_key(api_key)
            .secret_key(secret_key)
            .build();

        Ok(params)
    }
}

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BinanceSpotClient {
    params: Params,
    // outputs:
    //      0: SpotClient
    infra: NodeInfra,
}

impl BinanceSpotClient {
    pub(crate) fn try_new(node: Node) -> Result<Self> {
        let params = Params::try_from(&node)?;
        let mut infra = NodeInfra::new(node);

        let client = Client::builder()
            .api_key(&params.api_key)
            .secret_key(&params.secret_key)
            .build();

        let client_slot = Arc::new(Slot::<SpotClientKind>::new(client.into()));

        infra.port.set_output(0, client_slot)?;

        Ok(BinanceSpotClient { params, infra })
    }
}

impl NodePort for BinanceSpotClient {
    fn port(&self) -> &Port {
        &self.infra.port
    }

    fn port_mut(&mut self) -> &mut Port {
        &mut self.infra.port
    }
}

impl NodeExecutable for BinanceSpotClient {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<Node> for BinanceSpotClient {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
        BinanceSpotClient::try_new(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_binance_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BinanceSpotClient","params":["api_secret","secret"]}}"#;

        let node: Node = serde_json::from_str(json_str)?;
        let account = BinanceSpotClient::try_from(node)?;

        assert_eq!(account.params.api_key, "api_secret");
        assert_eq!(account.params.secret_key, "secret");

        Ok(())
    }

    // #[tokio::test]
    // async fn test_binance_account_execute() -> anyhow::Result<()> {
    //     let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret"]}}"#;

    //     let node: workflow::Node = serde_json::from_str(json_str)?;
    //     let account = BinanceSpotClient::try_from(node)?;

    //     // let _slot0 = account.port.output::<Arc<Mutex<SpotClient>>>(0)?;

    //     // let client = slot0.data().unwrap();

    //     // assert_eq!(client_information.client_type, "binance_client");
    //     // assert_eq!(
    //     //     client_information.data,
    //     //     Some(json!({"api_key": "api_secret", "secret_key": "secret", "market": "spot"}))
    //     // );

    //     Ok(())
    // }
}

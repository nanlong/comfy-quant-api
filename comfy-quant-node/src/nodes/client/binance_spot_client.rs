use crate::{
    node_core::{NodeCore, NodeCoreExt, NodeExecutable, NodeInfra, Slot},
    workflow::Node,
};
use anyhow::Result;
use bon::Builder;
use comfy_quant_exchange::client::{
    spot_client::binance_spot_client::BinanceSpotClient as Client, spot_client_kind::SpotClientKind,
};
use std::sync::Arc;

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BinanceSpotClient {
    params: Params,
    // outputs:
    //      0: SpotClient
    infra: NodeInfra,
}

impl NodeCore for BinanceSpotClient {
    fn node_infra(&self) -> &NodeInfra {
        &self.infra
    }

    fn node_infra_mut(&mut self) -> &mut NodeInfra {
        &mut self.infra
    }
}

impl BinanceSpotClient {
    pub(crate) fn try_new(node: Node) -> Result<Self> {
        let params = Params::try_from(&node)?;
        let infra = NodeInfra::new(node);

        Ok(BinanceSpotClient { params, infra })
    }
}

impl NodeExecutable for BinanceSpotClient {
    async fn setup(&mut self) -> Result<()> {
        let client = Client::builder()
            .api_key(&self.params.api_key)
            .secret_key(&self.params.secret_key)
            .build();

        let client_slot = Arc::new(Slot::<SpotClientKind>::new(client.into()));

        self.port_mut().set_output(0, client_slot)?;

        Ok(())
    }
}

impl TryFrom<Node> for BinanceSpotClient {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
        BinanceSpotClient::try_new(node)
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
#[allow(unused)]
pub(crate) struct Params {
    api_key: String,
    secret_key: String,
}

impl TryFrom<&Node> for Params {
    type Error = BinanceSpotClientError;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        if node.properties.prop_type != "client.BinanceSpotClient" {
            return Err(BinanceSpotClientError::PropertyTypeMismatch);
        }

        let [api_key, secret_key] = node.properties.params.as_slice() else {
            return Err(BinanceSpotClientError::ParamsFormatError);
        };

        let api_key = api_key
            .as_str()
            .ok_or(BinanceSpotClientError::ApiKeyError)?;

        let secret_key = secret_key
            .as_str()
            .ok_or(BinanceSpotClientError::SecretKeyError)?;

        let params = Params::builder()
            .api_key(api_key)
            .secret_key(secret_key)
            .build();

        Ok(params)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BinanceSpotClientError {
    #[error("Invalid property type, expected 'client.BinanceSpotClient'")]
    PropertyTypeMismatch,

    #[error("Invalid parameters format")]
    ParamsFormatError,

    #[error("Invalid api key")]
    ApiKeyError,

    #[error("Invalid secret key")]
    SecretKeyError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_binance_account() -> Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"client.BinanceSpotClient","params":["api_secret","secret"]}}"#;

        let node: Node = serde_json::from_str(json_str)?;
        let account = BinanceSpotClient::try_from(node)?;

        assert_eq!(account.params.api_key, "api_secret");
        assert_eq!(account.params.secret_key, "secret");

        Ok(())
    }

    // #[tokio::test]
    // async fn test_binance_account_execute() -> Result<()> {
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

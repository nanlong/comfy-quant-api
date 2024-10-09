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
    api_key: String,
    secret_key: String,
    market: String,
}

#[allow(unused)]
pub struct BinanceAccount {
    pub(crate) widget: Widget,
    pub(crate) ports: Ports,
}

impl BinanceAccount {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut ports = Ports::new();

        let client_information = ClientInformation::builder()
            .client_type("binance_client")
            .data(json!({"api_key": &widget.api_key, "secret_key": &widget.secret_key, "market": &widget.market}))
            .build();

        ports.add_output(
            0,
            Slot::<ClientInformation>::builder()
                .data(client_information)
                .build(),
        )?;

        Ok(BinanceAccount { widget, ports })
    }
}

impl NodePorts for BinanceAccount {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for BinanceAccount {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<workflow::Node> for BinanceAccount {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "account.binanceSubAccount" {
            anyhow::bail!("Try from workflow::Node to BinanceAccount failed: Invalid prop_type");
        }

        let [api_key, secret_key, market] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to BinanceAccount failed: Invalid params");
        };

        let api_key = api_key.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceAccount failed: Invalid api_key"
        ))?;

        let secret_key = secret_key.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceAccount failed: Invalid secret"
        ))?;

        let market = market.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceAccount failed: Invalid market"
        ))?;

        let widget = Widget::builder()
            .api_key(api_key)
            .secret_key(secret_key)
            .market(market)
            .build();

        BinanceAccount::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_binance_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret", "spot"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = BinanceAccount::try_from(node)?;

        assert_eq!(account.widget.api_key, "api_secret");
        assert_eq!(account.widget.secret_key, "secret");
        assert_eq!(account.widget.market, "spot");

        Ok(())
    }

    #[tokio::test]
    async fn test_binance_account_execute() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret", "spot"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = BinanceAccount::try_from(node)?;

        let slot0 = account.ports.get_output::<ClientInformation>(0)?;

        let client_information = slot0.data().unwrap();

        assert_eq!(client_information.client_type, "binance_client");
        assert_eq!(
            client_information.data,
            Some(json!({"api_key": "api_secret", "secret_key": "secret", "market": "spot"}))
        );

        Ok(())
    }
}

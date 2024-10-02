use crate::{
    base::{NodeExecutor, NodePorts, Ports, Slot},
    data::AccountKey,
    workflow,
};
use anyhow::Result;
use bon::Builder;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Widget {
    api_key: String,
    secret_key: String,
}

pub struct BinanceSubAccount {
    pub(crate) widget: Widget,
    pub(crate) ports: Ports,
}

impl BinanceSubAccount {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut ports = Ports::new();

        let account_key = AccountKey::builder()
            .api_key(&widget.api_key)
            .secret_key(&widget.secret_key)
            .build();

        ports.add_output(0, Slot::<AccountKey>::builder().data(account_key).build())?;

        Ok(BinanceSubAccount { widget, ports })
    }
}

impl NodePorts for BinanceSubAccount {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for BinanceSubAccount {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<workflow::Node> for BinanceSubAccount {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "account.binanceSubAccount" {
            anyhow::bail!("Try from workflow::Node to BinanceSubAccount failed: Invalid prop_type");
        }

        let [api_key, secret_key] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to BinanceSubAccount failed: Invalid params");
        };

        let api_key = api_key.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSubAccount failed: Invalid api_key"
        ))?;

        let secret_key = secret_key.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSubAccount failed: Invalid secret"
        ))?;

        let widget = Widget::builder()
            .api_key(api_key)
            .secret_key(secret_key)
            .build();

        BinanceSubAccount::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_binance_sub_account() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = BinanceSubAccount::try_from(node)?;

        assert_eq!(account.widget.api_key, "api_secret");
        assert_eq!(account.widget.secret_key, "secret");

        Ok(())
    }

    #[tokio::test]
    async fn test_binance_sub_account_execute() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let account = BinanceSubAccount::try_from(node)?;

        let slot0 = account.ports.get_output::<AccountKey>(0)?;

        let account_key = slot0.data().unwrap();

        assert_eq!(account_key.api_key, "api_secret");
        assert_eq!(account_key.secret_key, "secret");

        Ok(())
    }
}

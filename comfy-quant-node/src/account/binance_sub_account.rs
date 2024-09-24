use crate::{
    data::Account,
    traits::{NodeDataPort, NodeExecutor},
    workflow, DataPorts,
};
use anyhow::Result;
use tokio::sync::broadcast;

pub struct Widget {
    api_secret: String,
    secret: String,
}

impl Widget {
    pub fn new(api_secret: impl Into<String>, secret: impl Into<String>) -> Self {
        Widget {
            api_secret: api_secret.into(),
            secret: secret.into(),
        }
    }
}

pub struct BinanceSubAccount {
    pub(crate) widget: Widget,
    pub(crate) data_ports: DataPorts,
}

impl BinanceSubAccount {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut data_ports = DataPorts::new(0, 1);
        data_ports.add_output(0, broadcast::channel::<Account>(1).0)?;
        Ok(BinanceSubAccount { widget, data_ports })
    }

    async fn output0(&self) -> Result<()> {
        let tx = self.data_ports.get_output::<Account>(0)?.clone();

        let account = Account {
            api_secret: self.widget.api_secret.clone(),
            secret: self.widget.secret.clone(),
        };

        tokio::spawn(async move {
            while tx.receiver_count() > 0 {
                tx.send(account)?;
                break;
            }

            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

impl NodeDataPort for BinanceSubAccount {
    fn get_data_port(&self) -> Result<&DataPorts> {
        Ok(&self.data_ports)
    }

    fn get_data_port_mut(&mut self) -> Result<&mut DataPorts> {
        Ok(&mut self.data_ports)
    }
}

impl NodeExecutor for BinanceSubAccount {
    async fn execute(&mut self) -> Result<()> {
        self.output0().await?;
        Ok(())
    }
}

impl TryFrom<workflow::Node> for BinanceSubAccount {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "account.binanceSubAccount" {
            anyhow::bail!("Try from workflow::Node to BinanceSubAccount failed: Invalid prop_type");
        }

        let [api_secret, secret] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to BinanceSubAccount failed: Invalid params");
        };

        let api_secret = api_secret.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSubAccount failed: Invalid api_secret"
        ))?;

        let secret = secret.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSubAccount failed: Invalid secret"
        ))?;

        let widget = Widget::new(api_secret, secret);
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

        assert_eq!(account.widget.api_secret, "api_secret");
        assert_eq!(account.widget.secret, "secret");
        assert_eq!(account.data_ports.get_input_count(), 0);
        assert_eq!(account.data_ports.get_output_count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_binance_sub_account_execute() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"账户/币安子账户","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"account.binanceSubAccount","params":["api_secret","secret"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let mut account = BinanceSubAccount::try_from(node)?;

        let tx = account.data_ports.get_output::<Account>(0)?;
        let mut rx = tx.subscribe();

        account.execute().await?;

        let account = rx.recv().await?;
        assert_eq!(account.api_secret, "api_secret");
        assert_eq!(account.secret, "secret");

        Ok(())
    }
}

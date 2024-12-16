use super::{NodeContext, Port};
use crate::workflow::{Node, WorkflowContext};
use anyhow::{anyhow, Result};
use comfy_quant_base::{Exchange, Market, Symbol};
use rust_decimal::Decimal;
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeInfra {
    port: Port,
    node: Node,
}

impl NodeInfra {
    pub fn new(node: Node) -> Self {
        let port = Port::new();

        Self { port, node }
    }

    pub(crate) fn port(&self) -> &Port {
        &self.port
    }

    pub(crate) fn port_mut(&mut self) -> &mut Port {
        &mut self.port
    }

    pub(crate) fn node(&self) -> &Node {
        &self.node
    }

    pub(super) fn workflow_context(&self) -> Result<&Arc<WorkflowContext>> {
        self.node.workflow_context()
    }

    pub(super) fn node_context(&self) -> Result<NodeContext> {
        let context = self.workflow_context()?;

        Ok(NodeContext::new(
            context.cloned_db(),
            context.workflow_id(),
            self.node.id as i16,
            &self.node.properties.prop_type,
        ))
    }

    pub(super) async fn price(
        &self,
        exchange: &Exchange,
        market: &Market,
        symbol: &Symbol,
    ) -> Result<Decimal> {
        self.workflow_context()?
            .cloned_price_store()
            .read()
            .await
            .price(exchange, market, symbol)
            .ok_or_else(|| anyhow!("price not found"))
    }
}

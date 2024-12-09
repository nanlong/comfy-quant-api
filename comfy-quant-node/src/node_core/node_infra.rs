use super::Port;
use crate::workflow::{Node, WorkflowContext};
use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeInfra {
    pub port: Port,
    pub node: Node,
}

impl NodeInfra {
    pub fn new(node: Node) -> Self {
        let port = Port::new();

        Self { port, node }
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
        exchange: impl AsRef<str>,
        market: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> Result<Decimal> {
        self.workflow_context()?
            .cloned_price_store()
            .read()
            .await
            .price(exchange, market, symbol)
            .ok_or_else(|| anyhow!("price not found"))
    }
}

#[derive(Debug, Clone)]
pub struct NodeContext {
    pub db: Arc<PgPool>,
    pub workflow_id: String,
    pub node_id: i16,
    pub node_name: String,
}

impl NodeContext {
    pub fn new(
        db: Arc<PgPool>,
        workflow_id: impl Into<String>,
        node_id: i16,
        node_name: impl Into<String>,
    ) -> Self {
        Self {
            db,
            workflow_id: workflow_id.into(),
            node_id,
            node_name: node_name.into(),
        }
    }
}

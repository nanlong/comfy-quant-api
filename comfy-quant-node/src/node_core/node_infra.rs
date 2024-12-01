use super::Port;
use crate::workflow::{Node, WorkflowContext};
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeInfra {
    pub port: Port,
    pub node: Node,
}

impl NodeInfra {
    pub fn new(node: Node) -> Self {
        Self {
            port: Port::new(),
            node,
        }
    }

    pub fn workflow_context(&self) -> Result<&Arc<WorkflowContext>> {
        self.node.context()
    }

    pub fn node_context(&self) -> Result<NodeContext> {
        let context = self.node.context()?;

        Ok(NodeContext::new(
            context.cloned_db(),
            context.workflow_id(),
            self.node.node_id(),
            self.node.node_name(),
        ))
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

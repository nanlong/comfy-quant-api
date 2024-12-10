use std::sync::Arc;

use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct NodeContext {
    db: Arc<PgPool>,
    workflow_id: String,
    node_id: i16,
    node_name: String,
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

    pub fn db(&self) -> &PgPool {
        &self.db
    }

    pub fn cloned_db(&self) -> Arc<PgPool> {
        Arc::clone(&self.db)
    }

    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    pub fn node_id(&self) -> i16 {
        self.node_id
    }

    pub fn node_name(&self) -> &str {
        &self.node_name
    }
}

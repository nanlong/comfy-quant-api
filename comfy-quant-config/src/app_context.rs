use crate::setting::Setting;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;

#[derive(Debug)]
pub struct AppContext {
    pub setting: Setting,
    pub db: Arc<PgPool>,
}

impl AppContext {
    pub fn try_new() -> Result<Self> {
        let setting = Setting::try_new()?;

        let db = PgPoolOptions::new()
            .max_connections(20)
            .connect_lazy(&setting.database.url)?;

        Ok(Self {
            setting,
            db: Arc::new(db),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_context() -> Result<()> {
        let app_context = AppContext::try_new()?;
        assert_eq!(
            app_context.setting.database.url,
            "postgres://postgres:postgres@localhost:5432/comfy_quant_dev"
        );

        Ok(())
    }
}

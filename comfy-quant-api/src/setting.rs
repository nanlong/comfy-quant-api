use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Setting {
    pub(crate) debug: bool,
    pub(crate) database: Database,
}

impl Setting {
    pub fn try_new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or("dev".to_string());
        let database_url = dotenvy::var("DATABASE_URL").ok();

        let config = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/default.toml"))
            // Add in the current environment file
            // Default to 'dev' env
            // Note that this file is _optional_
            .add_source(File::with_name(&format!("config/{}.toml", run_mode)).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("config/local.toml").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            // You may also programmatically change settings
            .set_override_option("database.url", database_url)?
            .build()?;

        config.try_deserialize()
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub(crate) url: String,
}

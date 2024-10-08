use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::{env, path::Path};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Setting {
    pub(crate) debug: bool,
    pub(crate) database: Database,
}

impl Setting {
    pub fn try_new() -> Result<Self, ConfigError> {
        // 当前应用程序目录
        let app_dir = env!("CARGO_MANIFEST_DIR");
        // 从 .env 文件中获取运行模式
        let run_mode = dotenvy::var("RUN_MODE").unwrap_or("dev".to_string());
        // 从 .env 文件中获取数据库连接字符串
        let database_url = dotenvy::var("DATABASE_URL").ok();
        // 配置文件目录
        let config_dir = Path::new(&app_dir).join("../config");

        let config = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from(config_dir.join("default.toml")))
            // Add in the current environment file
            // Default to 'dev' env
            // Note that this file is _optional_
            .add_source(File::from(config_dir.join(format!("{}.toml", run_mode))).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::from(config_dir.join("local.toml")).required(false))
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

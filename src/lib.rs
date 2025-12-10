use serde::Deserialize;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::path::Path;

/// 数据库配置结构
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub database_url: String,
    pub max_connections: u32,
}

/// 顶级配置结构
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
}

impl Settings {
    /// 从 settings.toml 文件加载配置
    // pub fn new() -> Result<Self, config::ConfigError> {
    //     let s = config::Config::builder()
    //         .add_source(config::File::with_name("Settings"))
    //         .build()?;
    //
    //     s.try_deserialize()
    // }

    pub fn new<P: AsRef<Path>>(config_file_path: P) -> Result<Self, config::ConfigError> {
        let _path_str = config_file_path.as_ref().to_str().ok_or_else(|| {
            config::ConfigError::Message("Invalid config file path".to_string())
        })?;

        // config::File::from 路径更灵活，可以直接使用完整路径。
        let s = config::Config::builder()
            .add_source(config::File::from(config_file_path.as_ref()))
            .build()?;

        s.try_deserialize()
    }

    /// 根据配置创建 sqlx 数据库连接池
    pub async fn get_database_pool(&self) -> Result<Pool<MySql>, sqlx::Error> {
        MySqlPoolOptions::new()
            .max_connections(self.database.max_connections)
            .connect(&self.database.database_url)
            .await
    }
}



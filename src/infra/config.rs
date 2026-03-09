use serde::Deserialize;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::path::Path;
use tower_sessions_redis_store::fred::{
    clients::Pool as RedisPool,
    interfaces::ClientLike,
    prelude::Config,
};

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub database_url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
    pub pool_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogSettings {
    pub dir: String,
    pub file: String,
    pub level: String,
    pub max_history: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Settings {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub log: LogSettings,
    pub server: ServerSettings,
    pub s3: S3Settings,
}

impl Settings {
    pub fn new<P: AsRef<Path>>(config_file_path: P) -> Result<Self, config::ConfigError> {
        let _path_str = config_file_path
            .as_ref()
            .to_str()
            .ok_or_else(|| config::ConfigError::Message("Invalid config file path".to_string()))?;

        let s = config::Config::builder()
            .add_source(config::File::from(config_file_path.as_ref()))
            .build()?;

        s.try_deserialize()
    }

    pub async fn get_database_pool(&self) -> Result<Pool<MySql>, sqlx::Error> {
        MySqlPoolOptions::new()
            .max_connections(self.database.max_connections)
            .connect(&self.database.database_url)
            .await
    }

    pub async fn get_redis_pool(&self) -> Result<RedisPool, String> {
        let redis_config = Config::from_url(&self.redis.url).map_err(|e| e.to_string())?;

        let pool = RedisPool::new(redis_config, None, None, None, self.redis.pool_size)
            .map_err(|e| e.to_string())?;

        let connection_task = tokio::spawn(pool.connect());
        pool.wait_for_connect().await.map_err(|e| e.to_string())?;
        if connection_task.is_finished() {
            return Err("Redis connection task terminated prematurely".to_string());
        }

        let _: String = pool
            .ping(Some("ping".to_string()))
            .await
            .map_err(|e| format!("Redis pool PING failed: {}", e))?;

        Ok(pool)
    }

    pub fn get_s3_client(&self) -> aws_sdk_s3::Client {
        let creds = aws_sdk_s3::config::Credentials::new(
            &self.s3.access_key,
            &self.s3.secret_key,
            None,
            None,
            "static",
        );

        let config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .region(aws_sdk_s3::config::Region::new(self.s3.region.clone()))
            .endpoint_url(&self.s3.endpoint)
            .credentials_provider(creds)
            .force_path_style(true)
            .build();

        aws_sdk_s3::Client::from_conf(config)
    }
}

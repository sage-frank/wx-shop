use serde::Deserialize;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::path::Path;
use tower_sessions_redis_store::fred::{clients::Pool as RedisPool, interfaces::ClientLike, prelude::Config};

/// 数据库配置结构
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub database_url: String,
    pub max_connections: u32,
}

/// Redis 配置结构
#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
    pub pool_size: usize,
}

/// 日志配置结构
#[derive(Debug, Deserialize, Clone)]
pub struct LogSettings {
    pub dir: String,
    pub file: String,
    pub level: String,
}

/// 顶级配置结构
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub log: LogSettings,
}



impl Settings {
    /// 从 settings.toml 文件加载配置
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

    /// 根据配置创建 Redis 连接池
    pub async fn get_redis_pool(&self) -> Result<RedisPool, String> {
        let redis_config = Config::from_url(&self.redis.url).map_err(|e| e.to_string())?;
        // 显式设置 Redis 版本为 RESP2，因为 tower-sessions-redis-store 可能默认使用 RESP3 导致不兼容
        // redis_config.version = tower_sessions_redis_store::fred::types::RespVersion::RESP2;
        let pool = RedisPool::new(redis_config, None, None, None,self.redis.pool_size)
            .map_err(|e| e.to_string())?;
        let _ = pool.connect();
        pool.wait_for_connect().await.map_err(|e| e.to_string())?;
        let _: String = pool.ping(Some("ping".to_string())).await
            .map_err(|e| format!("Redis pool PING failed: {}", e.to_string()))?;
        Ok(pool)
    }
}

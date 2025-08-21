use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use validator::Validate;

// 配置模块入口
pub mod app;
pub mod auth;
pub mod openapi;
pub mod state;
pub mod redis;
pub mod database;
pub mod logging;
pub mod smtp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub server: app::ServerConfig,
    pub database: database::DatabaseConfig,
    pub redis: redis::RedisConfig,
    pub log: logging::LogConfig,
    pub smtp: smtp::SmtpConfig,
}


impl AppConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<AppConfig, anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// 创建默认配置文件
    pub fn get_default_config() -> String {
        let default_config = Self::default();
        let toml_content = toml::to_string_pretty(&default_config);
        toml_content.unwrap_or_else(|_| "".to_string())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        self.server.validate()?;
        self.database.validate()?;
        self.redis.validate()?;
        self.log.validate()?;
        Ok(())
    }

    /// 获取服务器地址
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "Axum Vue Admin".to_string(),
            server: app::ServerConfig::default(),
            database: database::DatabaseConfig::default(),
            redis:  redis::RedisConfig::default(),
            log: logging::LogConfig::default(),
            smtp: smtp::SmtpConfig::default(),
        }
    }
}
// # 日志配置

use anyhow::anyhow;
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Rotation {
    Hourly,
    Daily,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LogConfig {
    pub level: String,
    pub file: String,
    pub rotation: LogRotationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LogRotationConfig {
    /// 最大文件大小 (MB)
    pub max_size: u64,
    /// 保留文件数量
    pub max_files: u32,
    /// 是否按时间轮转
    pub daily: Rotation,
}



impl Default for LogConfig {
    fn default() -> Self {
        LogConfig{
            level: "info".to_string(),
            file: "log.log".to_string(),
            rotation: LogRotationConfig::default()
        }
    }
}


impl Default for LogRotationConfig {
    fn default() -> Self {
        LogRotationConfig{
            max_size: 100,
            max_files: 7,
            daily: Rotation::Daily,
        }
    }
}
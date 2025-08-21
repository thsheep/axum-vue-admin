// 应用配置

use serde::{Deserialize, Serialize};
use validator::Validate;

// --- 用于Redis的常量 ---
pub const REDIS_PUB_SUB_CHANNEL: &str = "policy_updates";
pub const BLACK_LIST_JTI: &str = "blacklist:jti";

// --------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub timeout_seconds: Option<u64>,
}


impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 9999,
            workers: Some(4),
            timeout_seconds: Some(30),
        }
    }
}

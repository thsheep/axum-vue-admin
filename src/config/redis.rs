use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RedisConfig {
    pub url: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
        }
    }
}
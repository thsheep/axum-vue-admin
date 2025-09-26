// # 数据库配置

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            url: "mysql://user:password@127.0.0.1:3306/database".to_owned(),
        }
    }
}
// # 认证配置（JWT、SSO等）

pub static JWT_SECRET: &str = "3488a63e1765035d386f05409663f55c83bfae3b3c61a932744b20ad14244dcf";
pub static ACCESS_TOKEN_EXPIRATION: i64 = 900; // 秒 15分钟

pub static REFRESH_TOKEN_EXPIRATION: i64 = 604800; // 秒 七天

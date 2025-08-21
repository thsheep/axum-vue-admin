use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize,Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls: bool,
}


impl Default for SmtpConfig {
    fn default() -> Self {
        SmtpConfig{
            host: "Mailtrap.io".to_string(),
            port: 2525,
            username: None,
            password: None,
            tls: false,
        }
    }
}
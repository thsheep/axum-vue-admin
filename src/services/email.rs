use core::str::FromStr;
use anyhow::Result;
use lettre::{message::Mailbox, Address, message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, SmtpTransport, Transport};
use tracing::{info, instrument};
use crate::config::smtp::SmtpConfig;
use crate::errors::app_error::AppError;

enum Mailer {
    Secure(AsyncSmtpTransport<Tokio1Executor>),
    Insecure(SmtpTransport),
}

pub struct EmailService {
    mailer: Mailer,
}

impl EmailService {

    pub fn new(config: &SmtpConfig) -> Self {
        let mailer = match config.tls {
            // --- 处理加密连接 (异步) ---
            true => {
                let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
                    .expect("Failed to create STARTTLS relay");
                builder = builder.port(config.port);
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    if !username.is_empty() {
                        let creds = Credentials::new(username.clone(), password.clone());
                        builder = builder.credentials(creds);
                    }
                }
                Mailer::Secure(builder.build())
            }
            // --- 处理不加密连接 (同步) ---
            false => {
                let mut builder = SmtpTransport::builder_dangerous(&config.host);
                builder = builder.port(config.port);
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    if !username.is_empty() {
                        let creds = Credentials::new(username.clone(), password.clone());
                        builder = builder.credentials(creds);
                    }
                }
                Mailer::Insecure(builder.build())
            }
        };

        Self { mailer }
    }

    // 发送邮件的方法
    #[instrument(skip(self, to, subject, body))]
    pub async fn send(&self, from:&str, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
        let address = Address::from_str(from).unwrap();
        let mailbox = Mailbox::new(Some("AxumVueAdmin".to_string()), address);
        let email = Message::builder()
            .from(mailbox)
            .to(to.parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(String::from(body))
            .unwrap();

        info!(recipient = %to, "正在发送邮件...");

        return match &self.mailer {
            Mailer::Secure(mailer) => {
                mailer.send(email).await?;
                info!("邮件发送成功");
                Ok(())
            }
            Mailer::Insecure(mailer) => {
                let mailer_clone = mailer.clone();
                tokio::task::spawn_blocking(move || mailer_clone.send(&email))
                    .await??; // 第一个 ? 处理 JoinError, 第二个 ? 处理 SmtpError
                info!("邮件发送成功");
                Ok(())
            }
        };
    }
}
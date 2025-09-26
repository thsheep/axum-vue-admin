use crate::errors::app_error::AppError;
use crate::services::cache::CacheService;
use crate::services::cedar_auth::CedarAuthService;
use crate::services::email::EmailService;
use crate::services::policy_link_manager::PolicyLinkManager;
use crate::utils::function::{
    load_active_policies_and_templates, load_active_schema, load_all_template_links,
};
use cedar_policy::{Entities, PolicySet, Schema};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::info;
use crate::config::AppConfig;

pub type SSESenders = Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: redis::Client,
    pub auth_service: Arc<CedarAuthService>,
    pub cache_service: Arc<CacheService>,
    pub email_service: Arc<EmailService>,
    pub sse_senders: SSESenders, // 这个SSE对象可以在全局Handler中对用户发送消息
    pub policy_link_manager: Arc<PolicyLinkManager>,
}

impl AppState {
    pub async fn new(
        config: &AppConfig
    ) -> Result<Self, AppError> {

        let mut opt = ConnectOptions::new(&config.database.url);
        opt.max_connections(100)
            .test_before_acquire(true)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(3600))
            .sqlx_logging(false);
        let db: DatabaseConnection = Database::connect(opt)
            .await
            .expect("Failed to connect to database");

        let redis = redis::Client::open(config.redis.url.clone()).expect("Failed to connect to redis client");


        let schema = load_active_schema(&db).await?;

        let cache_service = Arc::new(CacheService::new(redis.clone(), schema.clone()));
        let auth_service = Arc::new(CedarAuthService::new(
            cache_service.clone(),
            schema.clone(),
        ));

        let initial_policies = load_active_policies_and_templates(&db).await?;
        auth_service.update_policies_and_templates_in_cache(&initial_policies).await?;
        info!("已成功加载并缓存策略和模板。");

        let initial_link_records = load_all_template_links(&db).await?;
        auth_service.update_template_link_records_in_cache(&initial_link_records).await?;
        info!("成功加载并缓存 {} 个模板链接。", initial_link_records.len());

        let policy_link_manager = Arc::new(PolicyLinkManager::new(
            db.clone(),
            auth_service.clone(),
        ));

        let email_service = Arc::new(EmailService::new(&config.smtp));
        
        let app_state = Self {
            db,
            redis,
            auth_service,
            cache_service,
            email_service,
            sse_senders: Arc::new(Mutex::new(HashMap::new())),
            policy_link_manager
        };
        Ok(app_state)
    }
}

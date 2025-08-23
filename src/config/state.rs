use crate::errors::app_error::AppError;
use crate::services::cache::CacheService;
use crate::services::cedar_auth::CedarAuthService;
use crate::services::email::EmailService;
use crate::utils::function::{load_active_policies, load_active_schema};
use cedar_policy::{Entities, PolicySet, Schema};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use crate::config::AppConfig;

const USER_ENTITIES_CACHE_PREFIX: &str = "user_entities";

pub struct AppContextInner {
    pub policies: PolicySet,
    pub schema: Schema,
}

type SSESenders = Arc<Mutex<HashMap<u32, mpsc::Sender<String>>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: redis::Client,
    pub auth_service: Arc<CedarAuthService>,
    pub cache_service: Arc<CacheService>,
    pub email_service: Arc<EmailService>,
    pub sse_senders: SSESenders, // 这个SSE对象可以在全局Handler中对用户发送消息
}

impl AppState {
    pub async fn new(
        config: &AppConfig
    ) -> Result<Self, AppError> {

        let mut opt = ConnectOptions::new(&config.database.url);
        opt.max_connections(20)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(false);
        let db: DatabaseConnection = Database::connect(opt)
            .await
            .expect("Failed to connect to database");

        let redis = redis::Client::open(config.redis.url.clone()).expect("Failed to connect to redis client");


        let schema = load_active_schema(&db).await?;
        let policies = load_active_policies(&db).await?;

        let cache_service = Arc::new(CacheService::new(redis.clone(), schema.clone()));
        let auth_service = Arc::new(CedarAuthService::new(
            policies,
            schema,
            cache_service.clone(),
        ));

        let email_service = Arc::new(EmailService::new(&config.smtp));
        
        let app_state = Self {
            db,
            redis,
            auth_service,
            cache_service,
            email_service,
            sse_senders: Arc::new(Mutex::new(HashMap::new())),
        };
        Ok(app_state)
    }
}

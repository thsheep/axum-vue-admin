use crate::config::app::REDIS_PUB_SUB_CHANNEL;
use crate::config::state::AppState;
use crate::entity::{cedar_policy_set, cedar_schema};
use crate::errors::app_error::AppError;
use cedar_policy::{Policy, PolicyId, PolicySet, Schema};
use futures_util::StreamExt as _;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use std::str::FromStr;

use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(5);
const CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(10);


pub fn default_page() -> u64 {
    1
}

pub fn default_page_size() -> u64 {
    10
}

pub fn default_true() -> bool {
    true
}

pub fn default_false() -> bool {
    false
}



// 后台任务：监听Redis Pub/Sub的策略更新通知
pub async fn subscribe_to_policy_updates(state: AppState) {
    let mut consecutive_failures = 0;

    loop {
        info!("尝试订阅Redis频道 '{}'...", REDIS_PUB_SUB_CHANNEL);

        match establish_subscription(&state).await {
            Ok(()) => {
                consecutive_failures = 0;
                info!("Redis订阅会话结束，准备重新连接...");
            }
            Err(e) => {
                consecutive_failures += 1;
                error!(
                    "Redis订阅失败 (尝试 {}/{}): {}",
                    consecutive_failures, MAX_RETRY_ATTEMPTS, e
                );

                if consecutive_failures >= MAX_RETRY_ATTEMPTS {
                    error!("连续失败次数过多，延长重试间隔");
                    sleep(CONNECTION_RETRY_DELAY * consecutive_failures).await;
                } else {
                    sleep(CONNECTION_RETRY_DELAY).await;
                }
            }
        }
    }
}

async fn establish_subscription(state: &AppState) -> Result<(), AppError> {
    let pub_sub = state.redis.get_async_pubsub().await?;

    let (mut sink, mut stream) = pub_sub.split();

    // 订阅频道
    sink.subscribe(REDIS_PUB_SUB_CHANNEL).await?;

    info!("成功订阅Redis频道，等待通知...");

    // 处理消息循环
    while let Some(msg) = stream.next().await {
        let payload: String = match msg.get_payload() {
            Ok(p) => p,
            Err(e) => {
                warn!("无法从Redis消息中获取payload: {}，跳过此次更新。", e);
                continue; // 跳过当前消息，处理下一条
            }
        };

        if payload.is_empty() {
            warn!("收到的payload为空，跳过此次更新。");
            continue;
        }

        info!("收到策略更新通知: '{}'，开始重新加载...", payload);

        if let Err(e) = reload_policies_and_schema(state).await {
            error!("重新加载策略失败: {}", e);
            // 继续监听，不因单次加载失败而中断
        }
    }

    warn!("Redis消息流结束");
    Ok(())
}

pub async fn reload_policies_and_schema(state: &AppState) -> Result<(), AppError> {
    // 并发加载策略和Schema
    let (policies_result, schema_result) = tokio::join!(
        load_active_policies(&state.db),
        load_active_schema(&state.db)
    );

    let new_policies = policies_result?;
    let new_schema = schema_result?;
    state.auth_service.update_policies(new_policies).await?;
    state.auth_service.update_schema(new_schema).await?;
    info!("成功重新加载Cedar策略和Schema");
    Ok(())
}

pub async fn load_active_schema(db: &DatabaseConnection) -> Result<Schema, AppError> {
    info!("从数据库加载启用的 Cedar schema.............");
    let active_schema_model = cedar_schema::Entity::find()
        .one(db)
        .await?;

    match active_schema_model {
        Some(model) => {
            info!("找到schema (ID: {}). 解析中...", model.schema_id);
            let (schema, warning) = Schema::from_cedarschema_str(model.schema.as_str())?;
            warning.for_each(|w| {
                warn!("警告: {}", w);
            });
            Ok(schema)
        }
        None => {
            warn!("警告: 未找到有效的schema！从空Schema 开始");
            return Ok(Schema::from_json_value(serde_json::json!({}))?);
            // Err(AppError::SchemaError("CRITICAL: No active schema found in the database!".to_string()))
        }
    }
}

pub async fn load_active_policies(db: &DatabaseConnection) -> Result<PolicySet, AppError> {
    info!("从数据库加载有效的 Cedar policies...");
    let active_policy_models = cedar_policy_set::Entity::find()
        .filter(cedar_policy_set::Column::IsActive.eq(true))
        .all(db)
        .await?;

    if active_policy_models.is_empty() {
        warn!("警告：未找到有效策略。从空策略集开始。");
        return Ok(PolicySet::new());
    }

    let mut policies = PolicySet::new();

    for policy_model in active_policy_models {
        let policy_id = PolicyId::from_str(policy_model.policy_str_id.as_str())?;
        let policy = Policy::parse(Some(policy_id), policy_model.policy_text.as_str())?;
        policies.add(policy)?;
    }

    Ok(policies)
}

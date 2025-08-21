use crate::errors::app_error::AppError;
use cedar_policy::{Entities, Schema};
use moka::future::Cache;
use redis::{AsyncCommands, Client as RedisClient};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, instrument, warn};
use crate::utils::cedar_utils::entities2json;

const USER_ENTITIES_CACHE_PREFIX: &str = "user_entities";
const DEFAULT_LOCAL_CACHE_SIZE: u64 = 1000;
const DEFAULT_LOCAL_TTL_SECS: u64 = 300; // 5 minutes
const DEFAULT_REDIS_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Clone)]
pub struct CacheService {
    redis_client: RedisClient,
    local_cache: Cache<String, Entities>,
    schema: Arc<RwLock<Schema>>,
}

impl CacheService {
    pub fn new(redis_client: RedisClient, schema: Schema) -> Self {
        let local_cache = Cache::builder()
            .max_capacity(DEFAULT_LOCAL_CACHE_SIZE)
            .time_to_live(Duration::from_secs(DEFAULT_LOCAL_TTL_SECS))
            .build();

        Self {
            redis_client,
            local_cache,
            schema: Arc::new(RwLock::new(schema)),
        }
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_user_entities(&self, user_id: i32) -> Result<Option<Entities>, AppError> {
        let cache_key = format!("{}:{}", USER_ENTITIES_CACHE_PREFIX, user_id);

        // 1. 尝试从本地缓存获取
        if let Some(entities) = self.local_cache.get(&cache_key).await {
            // debug!("在本地缓存中找到的用户实体[UserID:{}]", user_id);
            return Ok(Some(entities));
        }

        // 2. 从Redis获取
        match self.get_from_redis(&cache_key).await {
            Ok(Some(entities)) => {
                // 存入本地缓存
                self.local_cache.insert(cache_key, entities.clone()).await;
                debug!("从 Redis 加载用户实体[UserID:{}]并缓存在本地", user_id);
                Ok(Some(entities))
            }
            Ok(None) => {
                debug!("Redis 中未找到用户实体");
                Ok(None)
            }
            Err(e) => {
                warn!("无法从 Redis 获取用户实体：{}", e);
                Ok(None)
            }
        }
    }

    #[instrument(skip(self, entities), fields(user_id = %user_id))]
    pub async fn cache_user_entities(
        &self,
        user_id: i32,
        entities: Entities,
    ) -> Result<(), AppError> {
        let cache_key = format!("{}:{}", USER_ENTITIES_CACHE_PREFIX, user_id);

        // 解析实体以验证格式并存入本地缓存
        let schema = self.schema.read().await;
        let parsed_entities = Entities::from_entities(entities, Some(&schema))?;
        let entities_json_str = entities2json(&parsed_entities)?;
        // 异步更新两个缓存层
        let redis_result = self.set_to_redis(&cache_key, entities_json_str.as_str()).await;
        self.local_cache
            .insert(cache_key.clone(), parsed_entities)
            .await;

        match redis_result {
            Ok(_) => {
                debug!("用户实体[{}]缓存成功", user_id);
                Ok(())
            }
            Err(e) => {
                error!("无法将用户实体[{}]缓存到 Redis：{}", user_id, e);
                Ok(())
            }
        }
    }

    #[instrument(skip(self), fields(username = %username))]
    pub async fn invalidate_user_entities(&self, username: &str) -> Result<(), AppError> {
        let cache_key = format!("{}:{}", USER_ENTITIES_CACHE_PREFIX, username);

        // 从本地缓存删除
        self.local_cache.invalidate(&cache_key).await;

        // 从Redis删除
        if let Err(e) = self.delete_from_redis(&cache_key).await {
            warn!("无法从 Redis 中删除用户实体[{}]：{}", username, e);
        }

        debug!("用户实体失效");
        Ok(())
    }

    pub async fn update_schema(&self, new_schema: Schema) {
        let mut schema = self.schema.write().await;
        *schema = new_schema;
        // 清空本地缓存，因为schema变更可能影响实体解析
        self.local_cache.invalidate_all();
        debug!("缓存架构已更新，本地缓存已清除");
    }

    // 获取缓存统计信息
    pub async fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            local_cache_size: self.local_cache.entry_count(),
            local_cache_capacity: DEFAULT_LOCAL_CACHE_SIZE,
        }
    }

    async fn get_from_redis(&self, key: &str) -> Result<Option<Entities>, AppError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let entities_json: Option<String> = conn.get(key).await?;

        match entities_json {
            Some(json) => {
                let schema = self.schema.read().await;
                let entities = Entities::from_json_str(&json, Some(&*schema))?;
                Ok(Some(entities))
            }
            None => Ok(None),
        }
    }

    async fn set_to_redis(&self, key: &str, value: &str) -> Result<(), AppError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let _: () = conn
            .set(key, value)
            .await?;
        Ok(())
    }

    async fn delete_from_redis(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub local_cache_size: u64,
    pub local_cache_capacity: u64,
}
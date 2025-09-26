use crate::errors::app_error::AppError;
use cedar_policy::{Entities, Schema};
use moka::future::Cache;
use redis::{AsyncCommands, Client as RedisClient};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, instrument, warn};
use crate::utils::cedar_utils::entities2json;

const DEFAULT_LOCAL_CACHE_SIZE: u64 = 1000;
const DEFAULT_LOCAL_TTL_SECS: u64 = 300; // 5 minutes
const DEFAULT_REDIS_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Clone)]
pub struct CacheService {
    redis_client: RedisClient,
    local_cache: Cache<String, String>,
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

    pub async fn get_entities(&self, cache_key: String) -> Result<Option<Entities>, AppError> {

        if let Some(cache_value) = self.local_cache.get(&cache_key).await {
            // debug!("在本地缓存中找到的用户实体[UserID:{}]", user_id);
            let schema = self.schema.read().await;
            let entities = Entities::from_json_str(&cache_value, Some(&*schema))?;
            return Ok(Some(entities));
        }

        match self.get_from_redis(&cache_key).await {
            Ok(Some(cache_value)) => {
                let schema = self.schema.read().await;
                let entities = Entities::from_json_str(&cache_value, Some(&*schema))?;
                self.local_cache.insert(cache_key.clone(), cache_value).await;
                debug!("从 Redis 加载实体[CacheKey:{}]并缓存在本地", cache_key);
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

    pub async fn cache_entities(
        &self,
        cache_key: String,
        entities: Entities,
    ) -> Result<(), AppError> {
        // 解析实体以验证格式并存入本地缓存
        let schema = self.schema.read().await;
        let parsed_entities = Entities::from_entities(entities, Some(&schema))?;
        let entities_json_str = entities2json(&parsed_entities)?;
        self.set_cache(cache_key, entities_json_str.as_str(), None).await?;
        Ok(())
    }
    
    pub async fn invalidate_user_entities(&self, cache_key: String) -> Result<(), AppError> {

        // 从本地缓存删除
        self.local_cache.invalidate(&cache_key).await;

        // 从Redis删除
        if let Err(e) = self.delete_from_redis(&cache_key).await {
            warn!("无法从 Redis 中删除实体[{}]：{}", cache_key, e);
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

    pub async fn get_cache(&self, cache_key: &str) -> Result<Option<String>, AppError> {

        if let Some(cache) = self.local_cache.get(cache_key).await {
                return Ok(Some(cache));
        }

        if let Some(cache) = self.get_from_redis(cache_key).await? {
            return Ok(Some(cache));
        }

        Ok(None)
    }

    /*
       非必要情况下，应该为每次缓存设置一个TTL.
       ttl_secs 应该大于 DEFAULT_LOCAL_TTL_SECS(这是本地缓存的时间) 值
    */
    pub async fn set_cache(&self, cache_key: String, cache_value: &str, ttl_secs: Option<u64>) -> Result<(), AppError> {
        let redis_result = self.set_to_redis(&cache_key, cache_value, ttl_secs).await;
        self.local_cache
            .insert(cache_key.clone(), cache_value.to_string())
            .await;

        match redis_result {
            Ok(_) => {
                debug!("实体[{}]缓存成功", cache_key);
                Ok(())
            }
            Err(e) => {
                error!("无法将实体[{}]缓存到 Redis：{}", cache_key, e);
                Ok(())
            }
        }
    }

    async fn get_from_redis(&self, key: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        let cache: Option<String> = conn.get(key).await?;
        Ok(cache)
    }

    async fn set_to_redis(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), AppError> {
        
        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        if let Some(ttl_secs) = ttl_secs {
            let _: () = conn
                .set_ex(key, value, ttl_secs)
                .await?;
        }
        
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
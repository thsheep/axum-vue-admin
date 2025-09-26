use std::collections::HashMap;
use std::str::FromStr;
use crate::errors::app_error::AppError;
use crate::services::cache::CacheService;
use cedar_policy::{Authorizer, Decision, Entities, PolicySet, Request, Schema, SlotId};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument, debug, error};
use crate::forbidden;
use crate::schemas::cedar_policy::{CedarContext, TemplateLinkRecord};
use crate::schemas::user::UserUUID;
use crate::utils::cedar_utils::{AuthAction, AuthorizationBuilder, ResourceType, POLICIES_AND_TEMPLATES_CACHE_KEY, TEMPLATE_LINKS_CACHE_KEY, USER_ENTITIES_CACHE_PREFIX};

#[derive(Clone)]
pub struct CedarAuthService {
    authorizer: Arc<Authorizer>,
    cache_service: Arc<CacheService>,
    schema: Arc<RwLock<Schema>>,
}

impl CedarAuthService {
    pub fn new(
        cache_service: Arc<CacheService>,
        schema: Schema,
    ) -> Self {
        tokio::spawn({
            let cache_service = cache_service.clone();
            let schema = schema.clone();
            async move {
                cache_service.update_schema(schema).await;
            }
        });

        Self {
            authorizer: Arc::new(Authorizer::new()),
            cache_service,
            schema: Arc::new(RwLock::new(schema)),
        }
    }

    pub async fn check_permission(
        &self,
        user_id: &UserUUID,
        context: CedarContext,
        action: AuthAction,
        resource: ResourceType,
    ) -> Result<bool, AppError> {
        let (request, resource_entities) = AuthorizationBuilder::new(user_id.clone(), context)
            .action(action)
            .resource(resource)
            .build()?;

        self.is_authorized(user_id, &request, resource_entities)
            .await
    }

    /// 带资源实体的授权检查
    pub async fn check_permission_with_entities(
        &self,
        user_id: &UserUUID,
        context: CedarContext,
        action: AuthAction,
        resource: ResourceType,
        resource_entities: Entities,
    ) -> Result<bool, AppError> {
        let (request, _) = AuthorizationBuilder::new(user_id.clone(), context)
            .action(action)
            .resource(resource)
            .resource_entities(resource_entities.clone())
            .build()?;

        self.is_authorized(user_id, &request, resource_entities)
            .await
    }

    pub async fn is_authorized(
        &self,
        user_id: &UserUUID,
        request: &Request,
        resource_entities: Entities,
    ) -> Result<bool, AppError> {
        // 从缓存获取用户实体
        let cache_key = format!("{}:{}", USER_ENTITIES_CACHE_PREFIX, user_id);

        let user_entities = self
            .cache_service
            .get_entities(cache_key)
            .await?
            .ok_or_else(||forbidden!(format!("UserID[{}] Entities Not Found", user_id)))?;


        let mut effective_policies = self.get_policies_and_templates_from_cache().await?
            .unwrap_or_else(PolicySet::new);

        let link_records = self.get_template_link_records_from_cache().await?
            .unwrap_or_default();

        if !link_records.is_empty() {
            for record in link_records {
                let mut values = HashMap::new();
                values.insert(SlotId::principal(), record.principal_uid);
                values.insert(SlotId::resource(), record.resource_uid);
                if let Err(e) = effective_policies.link(
                    record.template_uuid.clone(),
                    record.link_uuid.clone(),
                    values,
                ) {
                    warn!(
                        "无法将模板“{}”链接到链接“{}”：{}。跳过。",
                        record.template_uuid, record.link_uuid, e
                    );
                }
            }
        }
        // debug!("effective_policies: {}", effective_policies);
        // 合并资源实体
        let schema = self.schema.read().await;
        let combined_entities = user_entities.add_entities(
            resource_entities,
            Some(&schema),
        )?;
        // debug!("combined entities: {:?}", combined_entities);
        // 执行授权检查
        let response = self
            .authorizer
            .is_authorized(request, &effective_policies, &combined_entities);


        match response.decision() {
            Decision::Allow => {
                for policy_id in response.diagnostics().reason() {
                    if let Some(policy) = &effective_policies.policy(policy_id) {
                        debug!("UserID:{} 请求放行，原因：{:#?}",
                            user_id,
                            policy.annotation("annotation")
                            .unwrap_or("没有设置 @annotation"));
                    }
                }
                Ok(true)
            },
            Decision::Deny => {
                for policy_id in response.diagnostics().reason() {
                    if let Some(policy) = &effective_policies.policy(policy_id) {
                        debug!("UserID:{} 请求拒绝，原因：{:#?}",
                            user_id,
                            policy.annotation("annotation").unwrap_or("没有设置 @annotation"));
                        return Err(forbidden!("access denied"))
                    }
                }

                for error in response.diagnostics().errors() {
                    error!("错误: {}", error);
                }
                debug!("UserID {} 请求拒绝，原因：没有匹配到放行规则", user_id);
                Err(forbidden!("access denied[No Policy]".to_string()))
            }
        }
    }

    pub async fn get_policies_and_templates_from_cache(&self) -> Result<Option<PolicySet>, AppError> {
        if let Some(policy_string) = self.cache_service.get_cache(POLICIES_AND_TEMPLATES_CACHE_KEY).await? {
            let policy_set = PolicySet::from_str(&policy_string)?;
            Ok(Some(policy_set))
        } else {
            Ok(None)
        }
    }

    pub async fn get_template_link_records_from_cache(&self) -> Result<Option<Vec<TemplateLinkRecord>>, AppError> {
        if let Some(json_str) = self.cache_service.get_cache(TEMPLATE_LINKS_CACHE_KEY).await? {
            let records: Vec<TemplateLinkRecord> = serde_json::from_str(&json_str)?;
            Ok(Some(records))
        } else {
            Ok(None)
        }
    }

    pub async fn update_policies_and_templates_in_cache(&self, new_set: &PolicySet) -> Result<(), AppError> {
        let policy_string = new_set.to_string();
        self.cache_service.set_cache(POLICIES_AND_TEMPLATES_CACHE_KEY.to_string(), &policy_string, None).await
    }

    pub async fn update_template_link_records_in_cache(&self, records: &[TemplateLinkRecord]) -> Result<(), AppError> {
        let json_str = serde_json::to_string(records)?;
        self.cache_service.set_cache(TEMPLATE_LINKS_CACHE_KEY.to_string(), &json_str, None).await
    }

    pub async fn update_schema(&self, new_schema: Schema) {
        let mut schema_guard = self.schema.write().await;
        *schema_guard = new_schema.clone();
        self.cache_service.update_schema(new_schema).await;
        info!("CacheService Schema已更新。");
    }

    pub async fn get_schema_copy(&self) -> Schema {
        self.schema.read().await.clone()
    }
}
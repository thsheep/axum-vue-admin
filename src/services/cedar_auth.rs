use crate::errors::app_error::AppError;
use crate::services::cache::CacheService;
use cedar_policy::{Authorizer, Decision, Entities, PolicySet, Request, Schema};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument, debug, error};
use crate::forbidden;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::user::UserID;
use crate::utils::cedar_utils::{AuthAction, AuthorizationBuilder, ResourceType, USER_ENTITIES_CACHE_PREFIX};

pub struct AuthContextInner {
    pub policies: PolicySet,
    pub schema: Schema,
}

#[derive(Clone)]
pub struct CedarAuthService {
    inner: Arc<RwLock<AuthContextInner>>,
    authorizer: Arc<Authorizer>,
    cache_service: Arc<CacheService>,
}

impl CedarAuthService {
    pub fn new(
        policies: PolicySet,
        schema: Schema,
        cache_service: Arc<CacheService>,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AuthContextInner { policies, schema })),
            authorizer: Arc::new(Authorizer::new()),
            cache_service,
        }
    }

    pub async fn check_permission(
        &self,
        user_id: UserID,
        context: CedarContext,
        action: AuthAction,
        resource: ResourceType,
    ) -> Result<bool, AppError> {
        let (request, resource_entities) = AuthorizationBuilder::new(user_id, context)
            .action(action)
            .resource(resource)
            .build()?;

        self.is_authorized(user_id, &request, resource_entities)
            .await
    }

    /// 带资源实体的授权检查
    pub async fn check_permission_with_entities(
        &self,
        user_id: UserID,
        context: CedarContext,
        action: AuthAction,
        resource: ResourceType,
        resource_entities: Entities,
    ) -> Result<bool, AppError> {
        let (request, _) = AuthorizationBuilder::new(user_id, context)
            .action(action)
            .resource(resource)
            .resource_entities(resource_entities.clone())
            .build()?;

        self.is_authorized(user_id, &request, resource_entities)
            .await
    }

    pub async fn is_authorized(
        &self,
        user_id: UserID,
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

        // 合并资源实体
        let context = self.inner.read().await;
        let combined_entities = user_entities.add_entities(
            resource_entities,
            Some(&context.schema),
        )?;
        // debug!("combined entities: {:?}", combined_entities);
        // 执行授权检查
        let response = self
            .authorizer
            .is_authorized(request, &context.policies, &combined_entities);


        match response.decision() {
            Decision::Allow => {
                for policy_id in response.diagnostics().reason() {
                    if let Some(policy) = &context.policies.policy(policy_id) {
                        debug!("UserID:{} 请求被允许，PolicyID：{}", user_id, policy.id());
                    }
                }

                Ok(true)
            },
            Decision::Deny => {
                for policy_id in response.diagnostics().reason() {
                    if let Some(policy) = &context.policies.policy(policy_id) {
                        debug!("UserID:{} 请求被拒绝，PolicyID：{}", user_id, policy.id());
                        return Err(forbidden!("access denied".to_string()))
                    }
                }

                for error in response.diagnostics().errors() {
                    error!("错误: {}", error);
                }
                debug!("UserID{} 请求被拒绝，原因：没有匹配到放行规则", user_id);
                Err(forbidden!("access denied".to_string()))
            }
        }
    }

    pub async fn update_policies(&self, new_policies: PolicySet) -> Result<(), AppError> {
        let mut context = self.inner.write().await;
        context.policies = new_policies;
        info!("Policies 更新成功");
        Ok(())
    }

    pub async fn update_schema(&self, new_schema: Schema) -> Result<(), AppError> {
        let mut context = self.inner.write().await;
        context.schema = new_schema;
        info!("Schema 更新成功");
        Ok(())
    }

    pub async fn get_schema_copy(&self) -> Schema {
        self.inner.read().await.schema.clone()
    }
}
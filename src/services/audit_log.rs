// 审计日志路由
use crate::config::state::AppState;
use crate::errors::app_error::AppError;
use crate::entity::auditlog::{ActiveModel as AuditLogActiveModel, Entity as AuditLogEntity};
use sea_orm::{ActiveModelTrait, Set};
use crate::schemas::user::UserUUID;

#[derive(Clone)]
pub struct AuditLogService {
    app_state: AppState,
}

impl AuditLogService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub async fn log(
        &self,
        user_id: UserUUID,
        module: String,
        method: String,
        path: String,
        status: i32,
        response_time: i32,
    ) -> Result<(), AppError> {
        let audit_log = AuditLogActiveModel {
            user_id: Set(user_id),
            module: Set(module),
            method: Set(method),
            path: Set(path),
            status: Set(status),
            response_time: Set(response_time),
            created_at: Set(chrono::Local::now().naive_local()),
            updated_at: Set(chrono::Local::now().naive_local()),
            ..Default::default()
        };
        audit_log.insert(&self.app_state.db).await?;

        Ok(())
    }
}

// src/services/policy_link_manager.rs

use crate::errors::app_error::{AppError};
use crate::services::cedar_auth::{CedarAuthService};
use anyhow::Result;
use cedar_policy::{EntityUid, PolicyId};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait};
use std::collections::HashSet;
use std::sync::Arc;
use crate::entity::template_links;
use crate::not_found;
use crate::schemas::cedar_policy::TemplateLinkRecord;

#[derive(Clone)]
pub struct PolicyLinkManager {
    db: DatabaseConnection,
    auth_service: Arc<CedarAuthService>,
}

impl PolicyLinkManager {
    pub fn new(db: DatabaseConnection, auth_service: Arc<CedarAuthService>) -> Self {
        Self { db, auth_service }
    }

    pub async fn ensure_links_loaded(&self, link_ids: &[PolicyId]) -> Result<(), AppError> {
        if link_ids.is_empty() {
            return Ok(());
        }

        let cached_records = self.auth_service.get_template_link_records_from_cache().await?
            .unwrap_or_default();

        let cached_ids: HashSet<_> = cached_records.iter().map(|r| &r.link_uuid).collect();

        let ids_to_load: Vec<_> = link_ids.iter()
            .filter(|id| !cached_ids.contains(id))
            .cloned()
            .collect();

        if ids_to_load.is_empty() {
            return Ok(());
        }

        let new_records_from_db = self.load_link_records_from_db(&ids_to_load).await?;
        if new_records_from_db.is_empty() {
            return Err(not_found!("数据库中未找到一个或多个所需的策略链接。"));
        }

        let all_records = [cached_records, new_records_from_db].concat();
        self.auth_service.update_template_link_records_in_cache(&all_records).await?;

        Ok(())
    }

    pub async fn create_link(&self, record: TemplateLinkRecord) -> Result<(), AppError> {
        let new_link = template_links::ActiveModel {
            link_uuid: sea_orm::Set(record.link_uuid.to_string()),
            template_uuid: sea_orm::Set(record.template_uuid.to_string()),
            principal_uid: sea_orm::Set(record.principal_uid.to_string()),
            resource_uid: sea_orm::Set(record.resource_uid.to_string()),
            ..Default::default()
        };
        new_link.insert(&self.db).await?;

        let mut cached_records = self.auth_service.get_template_link_records_from_cache().await?
            .unwrap_or_default();

        cached_records.retain(|r| r.link_uuid != record.link_uuid);
        cached_records.push(record);

        self.auth_service.update_template_link_records_in_cache(&cached_records).await?;

        Ok(())
    }

    pub async fn delete_link(&self, link_uuid: &PolicyId) -> Result<(), AppError> {
        let res = template_links::Entity::delete_many()
            .filter(template_links::Column::LinkUuid.eq(link_uuid.to_string()))
            .exec(&self.db).await?;
        if res.rows_affected == 0 {
            return Err(not_found!(format!("未找到可删除的 link_uuid 为“{}”的链接。", link_uuid)));
        }

        let mut cached_records = self.auth_service.get_template_link_records_from_cache().await?
            .unwrap_or_default();

        let initial_len = cached_records.len();
        cached_records.retain(|r| &r.link_uuid != link_uuid);

        if cached_records.len() < initial_len {
            self.auth_service.update_template_link_records_in_cache(&cached_records).await?;
        }

        Ok(())
    }

    async fn load_link_records_from_db(&self, link_ids: &[PolicyId]) -> Result<Vec<TemplateLinkRecord>, AppError> {
        let string_ids: Vec<String> = link_ids.iter().map(|id| id.to_string()).collect();

        let link_models = template_links::Entity::find()
            .filter(template_links::Column::LinkUuid.is_in(string_ids))
            .all(&self.db)
            .await?;

        let link_records = link_models.into_iter().map(|model| {
            Ok(TemplateLinkRecord {
                link_uuid: model.link_uuid.parse()?,
                template_uuid: model.template_uuid.parse()?,
                principal_uid: model.principal_uid.parse()?,
                resource_uid: model.resource_uid.parse()?,
            })
        }).collect::<Result<Vec<TemplateLinkRecord>, AppError>>()?;

        Ok(link_records)
    }
}
use std::collections::{HashMap, HashSet};
use crate::entity::{
    departments::{Column as DepartmentColumn, Entity as DepartmentEntity, Relation as DepartmentRelation},
    roles::{Column as RoleColumn, Entity as RoleEntity, Relation as RoleRelation, Model as RoleModel},
    user_group_members::Column as UserGroupMemberColumn,
    user_groups::{Column as UserGroupColumn, Entity as UserGroupEntity, Relation as UserGroupRelation},
    user_roles::{Column as UserRoleColumn, Relation as UserRoleRelation},
    group_roles::{Column as GroupRoleColumn},
    users::{Column as UserColumn, Entity as UserEntity},
};
use crate::errors::app_error::AppError;
use crate::config::state::AppState;

use crate::schemas::me::{Info, Profile, UiPolicies};
use crate::schemas::user::{DeptResponse, GroupResponse};
use crate::schemas::{auth::CurrentUser};
use sea_orm::{ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait, JoinType, ModelTrait, QueryFilter, QuerySelect, QueryTrait, RelationTrait, Statement};
use tokio::task::JoinSet;
use crate::not_found;
use crate::schemas::cedar_policy::CedarContext;
use crate::services::user::UserService;
use crate::utils::cedar_utils::{AuthAction, ResourceType};


type UiKey = &'static str;
type UiActionMap = &'static [(UiKey, AuthAction)];

const UI_ACTIONS: UiActionMap = &[
    // Users
    ("menus:user_management", AuthAction::ViewUser),
    ("button:user_view", AuthAction::ViewUser),
    ("button:user_create", AuthAction::CreateUser),
    ("button:user_update", AuthAction::UpdateUser),
    ("button:user_delete", AuthAction::DeleteUser),
    // Groups
    ("menus:group_management", AuthAction::ViewGroup),
    ("button:group_view", AuthAction::ViewGroup),
    ("button:group_create", AuthAction::CreateGroup),
    ("button:group_update", AuthAction::UpdateGroup),
    ("button:group_delete", AuthAction::DeleteGroup),
    // Roles
    ("menus:role_management", AuthAction::ViewRole),
    ("button:role_view", AuthAction::ViewRole),
    ("button:role_create", AuthAction::ViewRole),
    ("button:role_update", AuthAction::UpdateRole),
    ("button:role_delete", AuthAction::DeleteRole),
    // Departments
    ("menus:dept_management", AuthAction::ViewDepartment),
    ("button:dept_view", AuthAction::ViewDepartment),
    ("button:dept_create", AuthAction::CreateDepartment),
    ("button:dept_update", AuthAction::UpdateDepartment),
    ("button:dept_delete", AuthAction::DeleteDepartment),
    // Policies
    ("menus:policies_management", AuthAction::ViewPolicy),
    ("button:policy_view", AuthAction::ViewPolicy),
    ("button:policy_create", AuthAction::CreatePolicy),
    ("button:policy_update", AuthAction::UpdatePolicy),
    ("button:policy_delete", AuthAction::DeletePolicy),
];


#[derive(Clone)]
pub struct MeService {
    app_state: AppState,
    user_service: UserService
}

impl MeService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state: app_state.clone(), user_service: UserService::new(app_state) }
    }

    pub async fn profile(&self,
                         current_user: CurrentUser,
                         context: CedarContext
    ) -> Result<Profile, AppError> {
        let ui_policies = self.ui_policies(&current_user, context).await?;
        let roles = self.roles(&current_user).await?;
        tracing::debug!("roles: {}", roles.len());
        let info = self.info(&current_user).await?;
        tracing::debug!("info: {:?}", info);
        let departments = self.department(&current_user).await?;
        tracing::debug!("departments: {:?}", departments);
        let groups = self.groups(&current_user).await?;
        tracing::debug!("groups: {:?}", groups);

        let profile = Profile {
            ui_policies,
            roles,
            info,
            departments,
            groups
        };
        Ok(profile)
    }

    async fn ui_policies(&self,
                         current_user: &CurrentUser,
                         context: CedarContext
    ) -> Result<UiPolicies, AppError> {

        let mut ui_policies = HashSet::with_capacity(UI_ACTIONS.len());;

        let mut join_set = JoinSet::new();

        for (ui_key, action) in UI_ACTIONS.iter() {
            let auth_service = self.app_state.auth_service.clone();
            let user = current_user.clone();
            let ctx = context.clone();
            let action = *action;
            let key = ui_key.to_string();

            join_set.spawn(async move {
                let has_permission = auth_service
                    .check_permission(user, ctx, action, ResourceType::UI(None))
                    .await?;

                Ok::<(String, bool), AppError>((key, has_permission))
            });
        }

        // 收集所有结果
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((key, has_permission))) => {
                    if has_permission {
                        ui_policies.insert(key);
                    }
                }
                Ok(Err(_e)) => continue, // 权限检查失败
                Err(e) => return Err(AppError::from(e)), // 任务执行失败
            }
        }

        Ok(ui_policies)
    }


    async fn department(
        &self,
        current_user: &CurrentUser,
    ) -> Result<Option<DeptResponse>, AppError> {

        let department = DepartmentEntity::find()
            .column_as(DepartmentColumn::DeptId, "id")
            .column_as(DepartmentColumn::Name, "name")
            .join(
                JoinType::InnerJoin,
                DepartmentRelation::Users.def(),
            )
            .filter(UserColumn::UserId.eq(current_user.user_id))
            .into_model::<DeptResponse>()
            .one(&self.app_state.db)
            .await?;

        Ok(department)
    }

    async fn groups(&self, current_user: &CurrentUser) -> Result<Vec<GroupResponse>, AppError> {
        let groups = UserGroupEntity::find()
            .column_as(UserGroupColumn::UserGroupId, "id")
            .column_as(UserGroupColumn::Name, "name")
            .join(
                JoinType::InnerJoin,
                UserGroupRelation::UserGroupMembers.def(),
            )
            .filter(UserGroupMemberColumn::UserId.eq(current_user.user_id))
            .into_model::<GroupResponse>()
            .all(&self.app_state.db)
            .await?;

        Ok(groups)
    }

    async fn info(&self, current_user: &CurrentUser) -> Result<Info, AppError> {
        let user = UserEntity::find_by_id(current_user.user_id)
            .into_model::<Info>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User not found".to_string()))?;
        Ok(user)
    }

    async fn roles(&self, current_user: &CurrentUser) -> Result<Vec<String>, AppError> {
        let role_names = self.user_service.get_user_role_models(current_user.user_id)
            .await?
            .into_iter()
            .map(|x| x.role_name)
            .collect::<Vec<String>>();

        Ok(role_names)
    }
}

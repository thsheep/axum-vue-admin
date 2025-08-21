use crate::errors::app_error::AppError;
use crate::schemas::department::DeptTreeNode;
use crate::schemas::user::{DeptResponse, GroupResponse, UserResponse};
use crate::entity::{
    departments::{
        Column as DepartmentColumn, Entity as DepartmentEntity, Model as DepartmentModel,
    },
    departments::{
        Column as DeptColumn, Entity as DeptEntity,
    },
    user_group_members::{Column as UserGroupMemberColumn},
    user_groups::{Entity as UserGroupEntity, Relation as UserGroupRelation},
    users::Model as UserModel,
};
use async_recursion::async_recursion;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use std::collections::{HashMap, VecDeque};
// Services 用的公共工具函数


// 递归构建部门树
#[async_recursion]
pub async fn build_dept_tree(
    all_departments: &[DepartmentModel],
    parent_id: i32,
) -> Result<Vec<DeptTreeNode>, AppError> {
    let mut tree_nodes = Vec::new();

    

    // 筛选出当前父部门下的所有子部门
    for dept in all_departments.iter().filter(|d| d.parent_id == parent_id) {
        // 递归构建子部门树
        let children = build_dept_tree(all_departments, dept.dept_id).await?;

        tree_nodes.push(DeptTreeNode {
            id: dept.dept_id,
            name: dept.name.clone(),
            desc: dept.desc.clone(),
            order: dept.order,
            parent_id: dept.parent_id,
            children,
        });
    }

    Ok(tree_nodes)
}

// 获取当前用户所有的子部门id
pub async fn children_dept(
    db: &DatabaseConnection,
    parent_id: i32,
) -> Result<Vec<i32>, sea_orm::DbErr> {
    // 获取所有部门并按 order 排序
    let all_depts = DepartmentEntity::find()
        .columns([DepartmentColumn::DeptId, DepartmentColumn::ParentId])
        .order_by_asc(DepartmentColumn::Order)
        .all(db)
        .await?;

    // 初始化结果集（包含自身）
    let mut dept_ids = vec![parent_id];
    // 使用 VecDeque 作为队列（比 Vec 更适合 FIFO 操作）
    let mut to_visit = VecDeque::from([parent_id]);

    while let Some(current_id) = to_visit.pop_front() {
        // 查找当前部门的所有直接子部门
        let children: Vec<i32> = all_depts
            .iter()
            .filter(|dept| dept.parent_id == current_id)
            .map(|dept| dept.dept_id)
            .collect();

        // 添加到结果集和待访问队列
        dept_ids.extend(&children);
        to_visit.extend(children);
    }

    Ok(dept_ids)
}

// 组装用户信息(用户信息、角色信息、部门信息)
pub async fn assemble_user_info(
    db: &DatabaseConnection,
    users_with_dept: Vec<(UserModel, Option<DepartmentModel>)>,
) -> Result<Vec<UserResponse>, AppError> {
    // 2. 收集所有用户ID
    let user_ids: Vec<i32> = users_with_dept
        .iter()
        .map(|(user, _)| user.user_id)
        .collect();

    // 3. 批量查询所有用户的用户组信息
    let user_group_relations = crate::entity::user_group_members::Entity::find()
        .find_also_related(UserGroupEntity)
        .filter(UserGroupMemberColumn::UserId.is_in(user_ids))
        .all(db)
        .await?;

    // 4. 按用户ID分组用户组信息
    let mut user_groups_map: HashMap<i32, Vec<GroupResponse>> = HashMap::new();
    for (relation, group_opt) in user_group_relations {
        if let Some(group) = group_opt {
            user_groups_map
                .entry(relation.user_id)
                .or_insert_with(Vec::new)
                .push(GroupResponse {
                    id: group.user_group_id,
                    name: group.name,
                });
        }
    }
    
    // 5. 构建最终结果
    let result = users_with_dept
        .into_iter()
        .map(|(user, dept)| UserResponse {
            id: user.user_id,
            username: user.username.clone(),
            alias: user.alias.clone(),
            email: user.email.clone(),
            phone: user.phone.clone(),
            is_active: user.is_active,
            dept: dept.map(|d| DeptResponse {
                id: d.dept_id,
                name: d.name,
            }),
            groups: user_groups_map
                .get(&user.user_id)
                .cloned()
                .unwrap_or_default(),
            avatar: user.avatar.clone(),
            last_login: user.last_login,
        })
        .collect();
    Ok(result)
}

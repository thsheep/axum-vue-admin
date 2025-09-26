use crate::config::app::BLACK_LIST_JTI;
use crate::config::auth::{ACCESS_TOKEN_EXPIRATION, REFRESH_TOKEN_EXPIRATION};
// 认证相关路由（登录、SSO等）
use crate::config::state::AppState;
use crate::entity::{
    departments,
    roles::{Column as RoleColumn, Entity as RoleEntity, Relation as RoleRelation},
    user_roles::Column as UserRoleColumn,
    users::{ActiveModel as UserActiveModel, Column as UserColumn, Entity as UserEntity},
};
use crate::errors::app_error::AppError;
use crate::schemas::auth::{AuthResponse, Claims, Credentials, TokenType};
use crate::services::user::{get_user_entities, UserService};
use crate::utils::{
    jwt::{create_access_token, decode_token},
    crypto::verify_password,
    cedar_utils::USER_ENTITIES_CACHE_PREFIX
};
use crate::{not_found, unauthorized};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use chrono::{Duration, Utc};
use cookie::{time::Duration as CookieDuration, SameSite};
use redis::{AsyncCommands, RedisResult};
use sea_orm::JoinType::InnerJoin;
use sea_orm::{ActiveModelTrait, ColIdx, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, QueryTrait, RelationTrait, Set};
use crate::schemas::user::UserUUID;

#[derive(Clone)]
pub struct AuthService {
    app_state: AppState,
    user_service: UserService,
}

impl AuthService {
    pub fn new(app_state: AppState) -> Self {
        Self { 
            app_state: app_state.clone(),
            user_service: UserService::new(app_state.clone()),
        }
    }

    pub async fn authenticate(
        &self,
        jar: CookieJar,
        dto: Credentials,
    ) -> Result<(CookieJar, AuthResponse), AppError> {
        // 验证用户名和密码
        // 生成 JWT
        // 返回 JWT
        let user = UserEntity::find()
            .filter(UserColumn::Username.eq(&dto.username))
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User Not found".to_string()))?;

        let verified = verify_password(&dto.password, user.password.as_str())?;
        if !verified {
            return Err(unauthorized!("Invalid credentials".to_string()));
        }

        if !user.is_active {
            return Err(unauthorized!("User is inactive".to_string()));
        }

        let is_super_admin = RoleEntity::find()
            .join(InnerJoin, RoleRelation::UserRoles.def())
            .filter(UserRoleColumn::UserId.eq(user.user_id))
            .filter(RoleColumn::RoleName.eq("SuperAdmin"))
            .count(&self.app_state.db)
            .await?;
        let is_super_admin = is_super_admin > 0;

        let dept_uuid = departments::Entity::find_by_id(user.dept_id)
            .select_only()
            .column(departments::Column::DeptUuid)
            .into_tuple::<String>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("Not joined the department".to_string()))?;

        let expires = Utc::now() + Duration::seconds(ACCESS_TOKEN_EXPIRATION);
        let payload = Claims {
            sub: user.user_uuid.clone(),
            jti: uuid::Uuid::new_v4(),
            iat: Utc::now().timestamp() as u64,
            exp: expires.timestamp() as u64,
            name: user.username.clone(),
            dept_id: dept_uuid.clone(),
            token_type: TokenType::Access,
            is_super_admin,
        };
        let access_token = create_access_token(payload).unwrap();

        let expires = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRATION);
        let payload = Claims {
            sub: user.user_uuid.clone(),
            jti: uuid::Uuid::new_v4(),
            iat: Utc::now().timestamp() as u64,
            exp: expires.timestamp() as u64,
            name: user.username.clone(),
            dept_id: dept_uuid,
            token_type: TokenType::Refresh,
            is_super_admin,
        };

        let refresh_token = create_access_token(payload).unwrap();

        let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
            .path("/api/v1/auth")
            .max_age(CookieDuration::seconds(604800))
            .same_site(SameSite::Strict)
            .http_only(true)
            .secure(true)
            .build();

        let _ = &self.cache_user_entities(user.user_uuid.clone()).await?;


        // 更新用户最后登录时间
        let user = UserActiveModel {
            user_id: Set(user.user_id),
            last_login: Set(Some(Utc::now().naive_local())),
            ..Default::default()
        };

        user.update(&self.app_state.db).await?;

        let auth_response = AuthResponse {
            access_token,
            username: dto.username,
        };
        Ok((jar.add(refresh_cookie), auth_response))
    }

    // 刷新 JWT
    pub async fn refresh(&self, jar: CookieJar) -> Result<(CookieJar, AuthResponse), AppError> {
        let refresh_token_str = jar
            .get("refresh_token")
            .map(|cookie| cookie.value().to_string())
            .ok_or(unauthorized!("Refresh token not found".to_string()))?;

        let refresh_claims = decode_token(refresh_token_str.as_str())?;

        if refresh_claims.token_type != TokenType::Refresh {
            return Err(unauthorized!("Invalid refresh token".to_string()));
        }
        // 检查这个Refresh Token是否在黑名单中
        let mut redis_conn = self
            .app_state
            .redis
            .get_multiplexed_async_connection()
            .await?;
        let is_blacklisted: bool = redis_conn
            .exists(format!("{}:{}", BLACK_LIST_JTI, refresh_claims.jti))
            .await?;
        if is_blacklisted {
            return Err(unauthorized!(
                "Refresh token is blacklisted".to_string(),
            ));
        }

        // 签发新的JWT
        let expires = Utc::now() + Duration::seconds(ACCESS_TOKEN_EXPIRATION);
        let new_claims = Claims {
            sub: refresh_claims.sub.clone(),
            jti: uuid::Uuid::new_v4(),
            iat: Utc::now().timestamp() as u64,
            exp: expires.timestamp() as u64,
            name: refresh_claims.name.clone(),
            dept_id: refresh_claims.dept_id.clone(),
            token_type: TokenType::Access,
            is_super_admin: refresh_claims.is_super_admin,
        };
        let new_access_token = create_access_token(new_claims).unwrap();

        let access_ttl = refresh_claims.exp - Utc::now().timestamp() as u64;
        // 将旧Refresh Token 添加黑名单
        let _: RedisResult<()> = redis_conn
            .set_ex(
                format!("{}:{}", BLACK_LIST_JTI, refresh_claims.jti),
                true,
                access_ttl,
            )
            .await;

        let expires = Utc::now() + Duration::minutes(REFRESH_TOKEN_EXPIRATION);
        let new_claims = Claims {
            sub: refresh_claims.sub,
            jti: uuid::Uuid::new_v4(),
            iat: Utc::now().timestamp() as u64,
            exp: expires.timestamp() as u64,
            name: refresh_claims.name.clone(),
            dept_id: refresh_claims.dept_id,
            token_type: TokenType::Refresh,
            is_super_admin: refresh_claims.is_super_admin,
        };
        let new_refresh_token = create_access_token(new_claims).unwrap();

        let new_refresh_cookie = Cookie::build(("refresh_token", new_refresh_token))
            .path("/api/v1/auth")
            .max_age(CookieDuration::seconds(604800))
            .same_site(SameSite::Strict)
            .http_only(true)
            .secure(true)
            .build();

        let auth_response = AuthResponse {
            access_token: new_access_token,
            username: refresh_claims.name,
        };

        Ok((jar.add(new_refresh_cookie), auth_response))
    }

    pub async fn logout(&self,
                        jar: CookieJar,
                        auth_header: Authorization<Bearer>,
    ) -> Result<CookieJar, AppError> {
        let mut redis_conn = self
            .app_state
            .redis
            .get_multiplexed_async_connection()
            .await?;

        // 设置 Access Token 过期
        let access_token_str = auth_header.token();
        let claims = decode_token(access_token_str)?;
        let ttl = claims.exp.saturating_sub(Utc::now().timestamp() as u64);
        if ttl > 0 {
            let _: RedisResult<()> = redis_conn
                .set_ex(format!("{}:{}", BLACK_LIST_JTI, claims.jti), true, ttl)
                .await;
        }
        
        // 设置 Refresh Token 过期
        if let Some(cookie) = jar.get("refresh_token") {
            let claims = decode_token(cookie.value())?;
            let ttl = claims.exp.saturating_sub(Utc::now().timestamp() as u64);
            if ttl > 0 {
                let _: RedisResult<()> = redis_conn
                    .set_ex(format!("{}:{}", BLACK_LIST_JTI, claims.jti), true, ttl)
                    .await;
            }
        }
        // 创建一个立即过期的cookie来删除客户端的cookie
        // let removal_cookie = Cookie::build(("refresh_token", ""))
        //     .path("/api/auth")
        //     .same_site(SameSite::Lax)
        //     .max_age(CookieDuration::ZERO)
        //     .build();
        
        Ok(jar.remove(Cookie::from("refresh_token")))
    }

    // 当前用户的 Entities 不应该过期; 不然速度太慢了.
    async fn cache_user_entities(&self, user_id: UserUUID) -> Result<(), AppError> {
        let cache_key = format!("{}:{}", USER_ENTITIES_CACHE_PREFIX, user_id);
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let user_entities = get_user_entities(&self.app_state.db, user_id, &schema).await?;
        self.app_state
            .cache_service
            .cache_entities(cache_key, user_entities)
            .await?;

        Ok(())
    }
    
}

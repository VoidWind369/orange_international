use crate::orange::{Clan, ClanUser};
use crate::system::UserInfo;
use crate::system::role::Role;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use std::fmt::{Display, Formatter};
use uuid::Uuid;
use void_log::{log_error, log_info, log_warn};

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: Option<i16>,
    pub code: Option<String>,
    pub phone: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n - Name: {}\n - Email: {}\n - Code: {}\n - Phone: {}\n - Create: {}\n - Update: {}",
            self.id.as_ref().unwrap_or(&Uuid::default()),
            self.name.as_ref().unwrap_or(&String::new()),
            self.email.as_ref().unwrap_or(&String::new()),
            self.code.as_ref().unwrap_or(&String::new()),
            self.phone.as_ref().unwrap_or(&String::new()),
            self.create_time.to_rfc2822(),
            self.update_time.to_rfc2822(),
        )
    }
}

impl User {
    fn get_password_hash(&self) -> String {
        // 密码Hash加密
        let salt = SaltString::generate();
        let argon2 = Argon2::default();
        argon2
            .hash_password(&self.password.clone().unwrap().as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from public.user").fetch_all(pool).await
    }

    pub async fn select_search(pool: &Pool<Postgres>, text: &str) -> Result<Vec<Self>, Error> {
        let text = format!("%{text}%");
        query_as("select * from public.user where email like $1 or code like $1 or name like $1")
            .bind(text)
            .fetch_all(pool)
            .await
    }

    pub async fn select(pool: &Pool<Postgres>, id: Uuid) -> Result<Self, Error> {
        query_as("select * from public.user where id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        let password = self.get_password_hash();
        query("insert into public.user values(DEFAULT, $1, $2, $3, $4, $5, $6, $6, $7)")
            .bind(&self.name)
            .bind(&self.email)
            .bind(&self.status)
            .bind(&self.code)
            .bind(&self.phone)
            .bind(now)
            .bind(password)
            .execute(pool)
            .await
    }

    pub async fn update(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update public.user set name = $1, email = $2, phone = $3, update_time = $4 where id = $5")
            .bind(&self.name)
            .bind(&self.email)
            .bind(&self.phone)
            .bind(now)
            .bind(&self.id)
            .execute(pool)
            .await
    }

    pub async fn update_status(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update public.user set status = $1, update_time = $2 where id = $3")
            .bind(&self.status)
            .bind(now)
            .bind(&self.id)
            .execute(pool)
            .await
    }

    pub async fn update_password(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        let password = self.get_password_hash();
        query("update public.user set password = $1, update_time = $2 where id = $3")
            .bind(password)
            .bind(now)
            .bind(&self.id)
            .execute(pool)
            .await
    }

    pub async fn delete(pool: &Pool<Postgres>, id: Uuid) -> Result<PgQueryResult, Error> {
        UserGroup::delete_user(id, pool).await?;
        ClanUser::delete_user(id, pool).await?;
        query("delete from public.user where id = $1")
            .bind(id)
            .execute(pool)
            .await
    }

    async fn verify_password(
        &self,
        password: &str,
        clans: Vec<Clan>,
        roles: Vec<Role>,
    ) -> Option<UserInfo> {
        let argon2 = Argon2::default();
        // 验证密码
        if let Ok(parsed_hash) = PasswordHash::new(&self.password.clone().unwrap()) {
            log_info!("用户数据校验");
            if let Err(e) = argon2.verify_password(password.as_bytes(), &parsed_hash) {
                // 校验失败
                log_warn!("Login failed: {e}");
                None
            } else {
                // 校验成功
                let timestamp = Utc::now().timestamp();
                let id = &self.id.unwrap();
                let code = &self.code.clone().unwrap();

                let token = format!("{}{}{}", timestamp, id, code);

                // 生成登录信息
                let user_info = UserInfo::new(self.clone(), token, clans, roles);
                // 存Redis
                user_info.set_user(3600).await;
                Some(user_info)
            }
        } else {
            log_error!("数据库密码存储错误");
            None
        }
    }

    pub async fn verify_login(&self, pool: &Pool<Postgres>) -> Option<UserInfo> {
        // 查用户
        let data_user = query_as::<_, User>(
            "select * from public.user where (email = $1 or code = $1) and status = 1",
        )
        .bind(&self.email)
        .fetch_one(pool)
        .await
        .unwrap_or_default();
        log_info!("{}", &data_user);
        // 查部落
        let user_clans = data_user.user_clans(pool).await.unwrap();
        // 查权限组
        let user_groups = data_user.user_groups(pool).await.unwrap();
        // 查权限
        let mut user_roles = vec![];
        for user_group in user_groups {
            let mut gr = user_group.group_roles(pool).await.unwrap();
            user_roles.append(&mut gr)
        }
        // 去重
        user_roles.dedup();
        // 通过查到的用户数据校验
        data_user
            .verify_password(&self.password.clone().unwrap(), user_clans, user_roles)
            .await
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct UserGroup {
    pub user_id: Uuid,
    pub group_id: Uuid,
}

impl UserGroup {
    pub async fn select(&self, pool: &Pool<Postgres>) -> Result<Self, Error> {
        query_as("select * from public.user_group where user_id = $1 and group_id = $2")
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("insert into public.user_group values ($1, $2)")
            .bind(self.user_id)
            .bind(self.group_id)
            .execute(pool)
            .await
    }

    pub async fn delete(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from public.user_group where user_id = $1 and group_id = $2")
            .bind(self.user_id)
            .bind(self.group_id)
            .execute(pool)
            .await
    }

    pub async fn delete_user(user_id: Uuid, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from public.user_group where user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
    }
}

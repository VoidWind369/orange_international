use crate::orange::{Clan, ClanUser};
use crate::system::Group;
use crate::system::UserInfo;
use crate::util::Config;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
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
    password: Option<String>,
}

impl User {
    fn get_password_hash(&self) -> String {
        // 密码Hash加密
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(&self.password.clone().unwrap().as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from public.user").fetch_all(pool).await
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
        query("update public.user set name = $1, email = $2, status = $3, phone = $4, updated_time = $5, where id = $6")
            .bind(&self.name)
            .bind(&self.email)
            .bind(&self.status)
            .bind(&self.phone)
            .bind(now)
            .bind(&self.id)
            .execute(pool)
            .await
    }

    pub async fn update_password(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        let password = self.get_password_hash();
        query("update public.user set password = $1, set update_time = $2 where id = $3")
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
        groups: Vec<Group>,
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
                let user_info = UserInfo::new(self.clone(), token, clans, groups);
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
        let data_user =
            query_as::<_, User>("select * from public.user where email = $1 or code = $1")
                .bind(&self.email)
                .fetch_one(pool)
                .await
                .unwrap_or_default();
        log_info!("{:?}", &data_user);
        // 查部落
        let user_clans = data_user.user_clans(pool).await.unwrap();
        // 查权限
        let user_groups = data_user.user_groups(pool).await.unwrap();
        log_info!("{:?}", &user_clans);
        log_info!("{:?}", &user_groups);
        // 通过查到的用户数据校验
        data_user
            .verify_password(&self.password.clone().unwrap(), user_clans, user_groups)
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
        query_as("select * from orange.user_group where user_id = $1 and group_id = $2")
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("insert into orange.user_group values ($1, $2)")
            .bind(self.user_id)
            .bind(self.group_id)
            .execute(pool)
            .await
    }

    pub async fn delete(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from orange.user_group where user_id = $1 and group_id = $2")
            .bind(self.user_id)
            .bind(self.group_id)
            .execute(pool)
            .await
    }

    pub async fn delete_user(user_id: Uuid, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from orange.user_group where user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
    }
}

#[tokio::test]
async fn test() {
    let config = Config::get().await;
    let pool = config.get_database().get().await;
    let user = User {
        name: Option::from("管理员1".to_string()),
        email: Option::from("mzx1@orgvoid.top".to_string()),
        status: Some(1),
        code: Option::from("admin1".to_string()),
        phone: Option::from("".to_string()),
        password: Option::from("123456".to_string()),
        ..Default::default()
    };
    let a = user.insert(&pool).await.unwrap();
    log_info!("{}", a.rows_affected())
}

#[tokio::test]
async fn test2() {
    let config = Config::get().await;
    let pool = config.get_database().get().await;
    let users = User::select_all(&pool).await.unwrap();
    log_info!("{:?}", users);
    for user in users {
        let a = user.create_time.with_timezone(&Local);
        let a = a.format("%Y-%m-%d %H:%M:%S").to_string();
        log_info!("{a} {}", user.get_password_hash())
    }
}

#[tokio::test]
async fn test3() {
    let config = Config::get().await;
    let pool = config.get_database().get().await;
    let users = User {
        email: Option::from("mzx1@orgvoid.top".to_string()),
        password: Option::from("123456".to_string()),
        ..Default::default()
    };
    let a = users.verify_login(&pool).await;
    log_info!("{:?}", a);
}

#[tokio::test]
async fn delete() {
    let config = Config::get().await;
    let pool = config.get_database().get().await;
    query("delete from public.user where code = 'admin1'")
        .execute(&pool)
        .await
        .unwrap();
}

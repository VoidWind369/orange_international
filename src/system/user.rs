use crate::system::redis::UserInfo;
use crate::util::Config;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;
use void_log::{log_error, log_info};

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct User {
    id: Option<Uuid>,
    name: Option<String>,
    email: Option<String>,
    status: Option<i16>,
    code: Option<String>,
    phone: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    password: String,
}

impl User {
    fn get_password_hash(&self) -> String {
        // 密码Hash加密
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(&self.password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }
    pub async fn select(conn: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as::<_, User>("select * from public.\"user\"")
            .fetch_all(conn)
            .await
    }

    pub async fn insert(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        let password = self.get_password_hash();
        query("insert into public.\"user\" values(DEFAULT, $1, $2, $3, $4, $5, $6, $6, $7)")
            .bind(&self.name)
            .bind(&self.email)
            .bind(&self.status)
            .bind(&self.code)
            .bind(&self.phone)
            .bind(now)
            .bind(password)
            .execute(conn)
            .await
    }

    pub async fn update(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update public.user set name = $1, email = $2, status = $3, phone = $4, updated_time = $5, where id = $6")
            .bind(&self.name)
            .bind(&self.email)
            .bind(&self.status)
            .bind(&self.phone)
            .bind(now)
            .bind(&self.id)
            .execute(conn)
            .await
    }

    pub async fn update_password(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        let password = self.get_password_hash();
        query("update public.user set password = $1, set update_time = $2 where id = $3")
            .bind(password)
            .bind(now)
            .bind(&self.id)
            .execute(conn)
            .await
    }

    pub async fn delete(id: Uuid, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from public.user where id = $1")
            .bind(id)
            .execute(conn)
            .await
    }

    async fn verify_password(&self, password_hash: &str) -> bool {
        let argon2 = Argon2::default();
        // 验证密码
        if let Ok(parsed_hash) = PasswordHash::new(password_hash) {
            log_info!("用户数据校验");
            let verify = argon2
                .verify_password(&self.password.as_bytes(), &parsed_hash)
                .is_ok();
            if verify {
                let timestamp = Utc::now().timestamp();
                let id = &self.id.unwrap();
                let code = &self.code.clone().unwrap();

                let token = format!("{}{}{}", timestamp, id, code);
                UserInfo::new(id.clone(), token).set_user(3600).await
            };
            verify
        } else {
            log_error!("数据库密码存储错误");
            false
        }
    }

    pub async fn verify_login(&self, conn: &Pool<Postgres>) -> bool {
        let data_user =
            query_as::<_, User>("select * from public.\"user\" where email = $1 or code = $1")
                .bind(&self.email)
                .fetch_one(conn)
                .await
                .unwrap_or_default();
        self.verify_password(&data_user.password).await
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
        password: "123456".to_string(),
        ..Default::default()
    };
    let a = user.insert(&pool).await.unwrap();
    log_info!("{}", a.rows_affected())
}

#[tokio::test]
async fn test2() {
    let config = Config::get().await;
    let pool = config.get_database().get().await;
    let users = User::select(&pool).await.unwrap();
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
        password: "123456".to_string(),
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

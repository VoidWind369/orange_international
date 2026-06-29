use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as, query_scalar};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct LoginLog {
    user_id: Uuid,
    #[serde(skip_deserializing)]
    login_time: DateTime<Utc>,
    address: String,
    code: Option<String>,
    name: Option<String>,
}

impl Display for LoginLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Login Log\n - Userid{}\n - Login Time: {}\n - Address: {}",
            &self.user_id, &self.login_time, &self.address
        )
    }
}

impl LoginLog {
    pub async fn new(user_id: Uuid, login_time: DateTime<Utc>, address: String) -> Self {
        Self {
            user_id,
            login_time,
            address,
            ..Default::default()
        }
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select ll.*, u.code, u.name from public.login_log ll, public.user u where ll.user_id = u.id and u.code not like '%admin%'")
            .fetch_all(pool)
            .await
    }

    pub async fn select_page(
        pool: &Pool<Postgres>,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<Self>, Error> {
        query_as("select ll.*, u.code, u.name from public.login_log ll, public.user u where ll.user_id = u.id and u.code not like '%admin%' order by login_time desc limit $1 offset $2")
            .bind(page_size)
            .bind(page_size * (page - 1))
            .fetch_all(pool)
            .await
    }

    /// # 分页数据总数
    pub async fn count(pool: &Pool<Postgres>) -> i64 {
        query_scalar("select count(id) from public.login_log")
            .fetch_one(pool)
            .await
            .unwrap_or_default()
    }

    pub async fn select(pool: &Pool<Postgres>, id: Uuid) -> Result<Vec<Self>, Error> {
        query_as("select ll.*, u.code, u.name from public.login_log ll, public.user u where ll.user_id = u.id and u.code not like '%admin%' and ll.user_id = $1")
            .bind(id)
            .fetch_all(pool)
            .await
    }

    pub async fn select_code_or_name(
        pool: &Pool<Postgres>,
        text: String,
    ) -> Result<Vec<Self>, Error> {
        let text = format!("%{text}%");
        query_as("select ll.*, u.code, u.name from public.login_log ll, public.user u where ll.user_id = u.id and u.code not like '%admin%' and (u.code like $1 or u.name like $1)")
            .bind(text)
            .fetch_all(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("insert into public.login_log values ($1, $2, $3)")
            .bind(self.user_id)
            .bind(self.login_time)
            .bind(&self.address)
            .execute(pool)
            .await
    }
}

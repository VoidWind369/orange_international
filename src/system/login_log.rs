use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{query, query_as, Error, FromRow, Pool, Postgres};
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

    pub async fn select(pool: &Pool<Postgres>, id: Uuid) -> Result<Vec<Self>, Error> {
        query_as("select ll.*, u.code, u.name from public.login_log ll, public.user u where ll.user_id = u.id and u.code not like '%admin%' and ll.user_id = $1")
            .bind(id)
            .fetch_all(pool)
            .await
    }

    pub async fn select_code_or_name(pool: &Pool<Postgres>, text: String) -> Result<Vec<Self>, Error> {
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

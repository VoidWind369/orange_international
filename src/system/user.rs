use crate::util;
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPoolOptions, PgQueryResult};
use sqlx::{Error, FromRow, Pool, Postgres};
use uuid::Uuid;
use void_log::log_info;

#[derive(Debug, Clone, PartialEq, Default, FromRow)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    status: i16,
    code: String,
    phone: String,
    created_time: NaiveDateTime,
    updated_time: NaiveDateTime,
    password: String,
}

impl User {
    async fn insert(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Local::now().naive_local();
        let sql = "insert into public.\"user\" values(DEFAULT, $1, $2, $3, $4, $5, $6, $6, $7)";
        sqlx::query(sql)
            .bind(&self.name)
            .bind(&self.email)
            .bind(&self.status)
            .bind(&self.code)
            .bind(&self.phone)
            .bind(now)
            .bind(&self.password)
            .execute(conn)
            .await
    }
}

#[tokio::test]
async fn test() {
    let config = util::Config::get().await;
    let pool = config.get_database().get().await;
    let user = User {
        name: "管理员".to_string(),
        email: "mzx@orgvoid.top".to_string(),
        status: 1,
        code: "admin".to_string(),
        phone: "".to_string(),
        password: "123456".to_string(),
        ..Default::default()
    };
    let a = user.insert(&pool).await.unwrap();
    log_info!("{}", a.rows_affected())
}

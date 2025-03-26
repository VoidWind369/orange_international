use crate::orange::Clan;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::postgres::PgSeverity::Log;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Round {
    id: Uuid,
    code: String,
    round_time: DateTime<Utc>,
    create_time: DateTime<Utc>,
}

impl Round {
    pub fn get_id(&self) -> Uuid {
        self.id
    }
    pub async fn select_last(conn: &Pool<Postgres>) -> Result<Self, Error> {
        query_as::<_, Self>("select * from orange.clan order by create_time desc limit 1")
            .fetch_one(conn)
            .await
    }

    pub async fn insert(time_str: &str, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let ndt = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").unwrap();
        let local_time = Local.from_local_datetime(&ndt).single().unwrap();
        let utc_time = local_time.with_timezone(&Utc);
        let code = utc_time.format("INTEL%Y%m%d").to_string();
        let now = Utc::now();

        query("insert into orange.round values(DEFAULT, $1, $2, $3)")
            .bind(code)
            .bind(utc_time)
            .bind(now)
            .execute(conn)
            .await
    }
}

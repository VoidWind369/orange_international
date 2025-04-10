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

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from orange.round").fetch_all(pool).await
    }

    pub async fn select_last(pool: &Pool<Postgres>) -> Result<Self, Error> {
        query_as::<_, Self>("select * from orange.round order by create_time desc limit 1")
            .fetch_one(pool)
            .await
    }

    pub async fn insert(time_str: &str, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let ndt = if let Ok(naive_date_time) =
            NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S")
        {
            naive_date_time
        } else {
            NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M").unwrap()
        };
        let local_time = Local.from_local_datetime(&ndt).single().unwrap();
        let utc_time = local_time.with_timezone(&Utc);
        let code = utc_time.format("INTEL%Y%m%d").to_string();
        let now = Utc::now();

        query("insert into orange.round values(DEFAULT, $1, $2, $3)")
            .bind(code)
            .bind(utc_time)
            .bind(now)
            .execute(pool)
            .await
    }
}

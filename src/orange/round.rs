use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
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

    pub fn get_code(&self) -> &str {
        &self.code
    }

    pub fn get_create_time(&self) -> DateTime<Utc> {
        self.create_time
    }

    pub async fn check_not_now(&self) -> bool {
        self.round_time > Utc::now()
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from orange.round order by create_time desc")
            .fetch_all(pool)
            .await
    }

    pub async fn select_last(pool: &Pool<Postgres>) -> Result<Self, Error> {
        query_as("select * from orange.round order by create_time desc limit 1")
            .fetch_one(pool)
            .await
    }

    pub async fn select_last2(pool: &Pool<Postgres>) -> Self {
        let last2 =
            query_as::<_, Self>("select * from orange.round order by create_time desc limit 2")
                .fetch_all(pool)
                .await
                .unwrap();
        if let Some(l) = last2.last() {
            l.clone()
        } else {
            Round::default()
        }
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
        let utc_time = local_time.to_utc().with_timezone(&Utc);
        let code = ndt.format("GLOBAL%Y%m%d").to_string();

        query("insert into orange.round values(DEFAULT, $1, $2, $3)")
            .bind(code)
            .bind(utc_time)
            .bind(Utc::now())
            .execute(pool)
            .await
    }
}

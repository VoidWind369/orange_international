use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;
use crate::orange::Clan;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct ClanPoint {
    pub clan_id: Uuid,
    pub point: i64,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub status: Option<i16>,
}

impl ClanPoint {
    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.clan_point values($1, $2, $3, $4)")
            .bind(&self.clan_id)
            .bind(&self.point)
            .bind(now)
            .bind(&self.status)
            .execute(pool)
            .await
    }

    pub async fn update_point(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update orange.clan_point set point = $1, update_time = $2 where clan_id = $3")
            .bind(&self.point)
            .bind(now)
            .bind(&self.clan_id)
            .execute(pool)
            .await
    }
}

impl Clan {
    pub async fn point_select(&self, pool: &Pool<Postgres>) -> Result<ClanPoint, Error> {
        query_as::<_,ClanPoint>("select * from clan_point where clan_id = $1")
            .bind(&self.id).fetch_one(pool).await
    }
}

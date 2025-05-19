use crate::orange::Clan;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;
use void_log::log_warn;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct ClanPoint {
    pub clan_id: Uuid,
    pub point: i64,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub status: Option<i16>,
    pub reward_point: i64,

    pub tag: Option<String>,
    pub name: Option<String>,
}

impl ClanPoint {
    pub fn new(clan_id: Uuid, point: i64) -> Self {
        Self {
            clan_id,
            point,
            status: Some(1),
            ..Default::default()
        }
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Self, Error> {
        query_as("select oc.tag, oc.name, ocp.* from orange.clan oc, orange.clan_point ocp where oc.id = ocp.clan_id")
            .fetch_one(pool)
            .await
    }

    pub async fn select(pool: &Pool<Postgres>, id: Uuid) -> Result<Self, Error> {
        query_as("select oc.tag, oc.name, ocp.* from orange.clan oc, orange.clan_point ocp where oc.id = ocp.clan_id and ocp.clan_id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.clan_point values($1, $2, $3, $3, $4)")
            .bind(&self.clan_id)
            .bind(&self.point)
            .bind(now)
            .bind(&self.status)
            .execute(pool)
            .await
    }

    pub async fn update_point(
        &self,
        pool: &Pool<Postgres>,
        add: i64,
    ) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update orange.clan_point set point = $1, update_time = $2 where clan_id = $3")
            .bind(&self.point + add)
            .bind(now)
            .bind(&self.clan_id)
            .execute(pool)
            .await
    }

    pub async fn update_reward_point(
        &self,
        pool: &Pool<Postgres>,
        reward_add: i64,
    ) -> Result<PgQueryResult, Error> {
        let reward_add = if self.reward_point > 5 {
            0
        } else {
            reward_add
        };
        let now = Utc::now();
        query("update orange.clan_point set reward_point = $1, update_time = $2 where clan_id = $3")
            .bind(&self.reward_point + reward_add)
            .bind(now)
            .bind(&self.clan_id)
            .execute(pool)
            .await
    }

    pub async fn delete(pool: &Pool<Postgres>, id: Uuid) -> Result<PgQueryResult, Error> {
        query("delete from orange.clan_point where clan_id = $1")
            .bind(id)
            .execute(pool)
            .await
    }

    pub async fn insert_or_update(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        if let Err(e) = Self::select(pool, self.clan_id).await {
            log_warn!("Select Null {e}");
            self.insert(pool).await
        } else {
            self.update_point(pool, 0).await
        }
    }
}

impl Clan {
    pub async fn point_select(&self, pool: &Pool<Postgres>) -> Result<ClanPoint, Error> {
        query_as("select ocp.*, oc.tag, oc.name from orange.clan_point ocp, orange.clan oc where ocp.clan_id = oc.id and ocp.clan_id = $1")
            .bind(&self.id)
            .fetch_one(pool)
            .await
    }
}

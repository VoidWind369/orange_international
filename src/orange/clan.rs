use crate::api;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{query, query_as, Error, FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Clan {
    pub id: Option<Uuid>,
    pub tag: Option<String>,
    pub name: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    status: Option<i16>,
    pub series_id: Option<Uuid>,
    is_intel: Option<bool>,
}

impl Clan {
    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as::<_, Clan>("select * from orange.clan")
            .fetch_all(pool)
            .await
    }

    pub async fn select_tag(
        tag: &str,
        is_intel: bool,
        pool: &Pool<Postgres>,
    ) -> Result<Self, Error> {
        query_as::<_, Clan>("select * from orange.clan where tag = $1 and is_intel = $2")
            .bind(tag)
            .bind(is_intel)
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.clan values(DEFAULT, $1, $2, $3, $3, $4, $5)")
            .bind(&self.tag)
            .bind(&self.name)
            .bind(now)
            .bind(&self.status)
            .bind(&self.series_id)
            .execute(pool)
            .await
    }

    pub async fn update(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update orange.clan set tag = $1, name = $2, updated_time = $3, status = $4, series_id = $5 where id = $6")
            .bind(&self.tag)
            .bind(&self.name)
            .bind(now)
            .bind(&self.status)
            .bind(&self.series_id)
            .bind(&self.id)
            .execute(pool)
            .await
    }

    pub async fn delete(id: Uuid, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from orange.clan where id = $1")
            .bind(id)
            .execute(pool)
            .await
    }

    /// # 接口自动更新
    pub async fn api_insert(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let tag = &self.tag.clone().unwrap_or_default();
        let clan = api::Clan::get(tag).await.api_to_orange();
        clan.insert(conn).await
    }
}

impl api::Clan {
    pub fn api_to_orange(&self) -> Clan {
        Clan {
            tag: (&self).tag.clone(),
            name: (&self).name.clone(),
            status: Some(1),
            ..Default::default()
        }
    }
}

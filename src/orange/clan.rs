use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error, FromRow, Pool, Postgres};
use sqlx::postgres::PgQueryResult;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Clan {
    id: Option<Uuid>,
    tag: Option<String>,
    name: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    status: Option<i16>,
    series_id: Option<Uuid>,
}

impl Clan {
    pub async fn select(conn: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as::<_, Clan>("select * from orange.clan")
            .fetch_all(conn)
            .await
    }

    pub async fn insert(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.clan values(DEFAULT, $1, $2, $3, $3, $4, $5)")
            .bind(&self.tag)
            .bind(&self.name)
            .bind(now)
            .bind(&self.status)
            .bind(&self.series_id)
            .execute(conn)
            .await
    }

    pub async fn update(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update orange.clan set tag = $1, name = $2, updated_time = $3, status = $4, series_id = $5 where id = $6")
            .bind(&self.tag)
            .bind(&self.name)
            .bind(now)
            .bind(&self.status)
            .bind(&self.series_id)
            .bind(&self.id)
            .execute(conn)
            .await
    }

    pub async fn delete(id: Uuid, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from orange.clan where id = $1")
            .bind(id)
            .execute(conn)
            .await
    }
}
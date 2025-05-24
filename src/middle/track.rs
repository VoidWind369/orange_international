use crate::api::{MiddleTrackApi, MiddleTrackApiDetails};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgQueryResult;
use sqlx::types::Json;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow)]
pub struct Track {
    pub id: Uuid,
    pub server: String,
    pub bz_total_score: i64,
    pub public_total_score: i64,
    pub details: Json<Vec<MiddleTrackApiDetails>>,
    pub summary: Vec<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub tag: String,
}

impl Track {
    pub fn self_to_api(self) -> MiddleTrackApi {
        MiddleTrackApi {
            server: self.server,
            bz_total_score: self.bz_total_score,
            public_total_score: self.public_total_score,
            details: self.details.0,
            summary: self.summary,
            tag: self.tag,
        }
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from middle.track").fetch_all(pool).await
    }

    pub async fn select_tag(pool: &Pool<Postgres>, tag: &str) -> Result<Self, Error> {
        let tag = format!("#{}", tag.replace("#", "").to_uppercase());
        query_as("select * from middle.track where tag = $1").bind(tag).fetch_one(pool).await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into middle.track values(DEFAULT, $1, $2, $3, $4, $5, $6, $6, $7)")
            .bind(&self.server)
            .bind(&self.bz_total_score)
            .bind(&self.public_total_score)
            .bind(&self.details)
            .bind(&self.summary)
            .bind(now)
            .bind(&self.tag)
            .execute(pool)
            .await
    }

    pub async fn update(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update middle.track set server = $1, bz_total_score = $2, public_total_score = $3, details = $4, summary = $5, update_time = $6 where tag = $7")
            .bind(&self.server)
            .bind(&self.bz_total_score)
            .bind(&self.public_total_score)
            .bind(&self.details)
            .bind(&self.summary)
            .bind(now)
            .bind(&self.tag).execute(pool).await
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
struct OperateLog {
    #[serde(skip_deserializing)]
    id: Uuid,
    round_id: Uuid,
    text: String,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    clan_id: Uuid,
    #[sqlx(skip)]
    reward_type: RewardType
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
enum RewardType {
    #[default]
    HitExternal,
    FaceBlack,
}

impl OperateLog {
    async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from orange.operate_logs o, orange.round r, orange.clan c where o.round_id = r.id and o.clan_id = c.id").fetch_all(pool).await
    }

    async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("insert into orange.operate_logs values (DEFAULT, $1, $2, $3, $4)")
            .bind(&self.round_id)
            .bind(&self.text)
            .bind(Utc::now())
            .bind(&self.clan_id)
            .execute(pool)
            .await
    }
}

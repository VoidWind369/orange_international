use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;
use crate::orange::clan_point::ClanPoint;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct OperateLog {
    #[serde(skip_deserializing)]
    pub id: Uuid,
    round_id: Uuid,
    text: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    clan_id: Uuid,
    #[sqlx(skip)]
    reward_type: RewardType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
enum RewardType {
    #[default]
    HitExternal, // 打虫减分
    FaceBlack, // 俩黑
    Penalty, // 处罚
}

impl OperateLog {
    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from orange.operate_logs o, orange.round r, orange.clan c where o.round_id = r.id and o.clan_id = c.id").fetch_all(pool).await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("insert into orange.operate_logs values (DEFAULT, $1, $2, $3, $4)")
            .bind(&self.round_id)
            .bind(&self.text)
            .bind(Utc::now())
            .bind(&self.clan_id)
            .execute(pool)
            .await
    }

    pub async fn new_reward(mut self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let clan_point = ClanPoint::select(pool, self.clan_id).await?;
        let text = match self.reward_type.clone() {
            RewardType::HitExternal => {
                if let Ok(q) = clan_point.update_reward_point(pool, 1).await {
                    q.rows_affected()
                } else { 
                    0
                };
                "打虫奖励+1".to_string() 
            },
            RewardType::FaceBlack => {
                if let Ok(q) = clan_point.update_reward_point(pool, 1).await {
                    q.rows_affected()
                } else {
                    0
                };
                "连输奖励+1".to_string() 
            },
            RewardType::Penalty => {
                if let Ok(q) = clan_point.update_reward_point(pool, -1).await {
                    q.rows_affected()
                } else {
                    0
                };
                "违规处罚-1".to_string() 
            },
        };
        self.text = Option::from(text);
        self.create_time = Utc::now();
        // 写入
        self.insert(pool).await
    }
}

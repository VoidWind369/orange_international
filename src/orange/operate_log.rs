use crate::orange::clan_point::ClanPoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct OperateLog {
    #[serde(skip_deserializing)]
    pub id: Uuid,
    pub round_id: Uuid,
    text: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    pub clan_id: Uuid,
    #[sqlx(skip)]
    reward_type: RewardType,
    #[serde(skip_deserializing)]
    tag: String,
    #[serde(skip_deserializing)]
    name: String,
    #[serde(skip_deserializing)]
    round_code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
enum RewardType {
    #[default]
    HitExternal, // 打虫减分
    FaceBlack, // 俩黑
    Penalty,   // 处罚1
    Penalty2,  // 处罚2
    Penalty3,  // 处罚3
}

impl OperateLog {
    pub fn is_reward_penalty(&self) -> bool {
        match &self.reward_type {
            RewardType::Penalty => true,
            _ => false,
        }
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select o.*, c.tag, c.name, r.code round_code from orange.operate_log o, orange.round r, orange.clan c where o.round_id = r.id and o.clan_id = c.id order by o.create_time desc").fetch_all(pool).await
    }

    pub async fn select_clan_round(&self, pool: &Pool<Postgres>) -> Result<Self, Error> {
        query_as("select * from orange.operate_log where clan_id = $1 and round_id = $2")
            .bind(&self.clan_id)
            .bind(self.round_id)
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("insert into orange.operate_log values (DEFAULT, $1, $2, $3, $4)")
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
            }
            RewardType::FaceBlack => {
                if let Ok(q) = clan_point.update_reward_point(pool, 1).await {
                    q.rows_affected()
                } else {
                    0
                };
                "脸黑安慰+1".to_string()
            }
            RewardType::Penalty => {
                if let Ok(q) = clan_point.update_reward_point(pool, -1).await {
                    q.rows_affected()
                } else {
                    0
                };
                "违规处罚-1".to_string()
            }
            RewardType::Penalty2 => {
                if let Ok(q) = clan_point.update_reward_point(pool, -2).await {
                    q.rows_affected()
                } else {
                    0
                };
                "违规处罚-2".to_string()
            }
            RewardType::Penalty3 => {
                if let Ok(q) = clan_point.update_reward_point(pool, -3).await {
                    q.rows_affected()
                } else {
                    0
                };
                "违规处罚-3".to_string()
            }
        };
        self.text = Option::from(text);
        self.create_time = Utc::now();
        // 写入
        self.insert(pool).await
    }
}

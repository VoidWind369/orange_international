use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct ClanPoint {
    clan_id: Uuid,
    point: i64,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
    status: Option<i16>,
}

impl ClanPoint {
    pub async fn insert(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.clan_point values($1, $2, $3, $4)")
            .bind(&self.clan_id)
            .bind(&self.point)
            .bind(now)
            .bind(&self.status)
            .execute(conn)
            .await
    }

    pub async fn update_point(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update orange.clan_point set point = $1, update_time = $2 where clan_id = $3")
            .bind(&self.point)
            .bind(now)
            .bind(&self.clan_id)
            .execute(conn)
            .await
    }
}

fn fight_point(self_clan_point: ClanPoint, rival_clan_point: ClanPoint) {
    let self_point = self_clan_point.point;
    let rival_point = rival_clan_point.point;
    let mut win_clan = ClanPoint::default();

    // self > rival
    if self_point > rival_point {
        win_clan = self_clan_point
    }

    // self < rival
    if self_point < rival_point {
        win_clan = rival_clan_point
    }
}

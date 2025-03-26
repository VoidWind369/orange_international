use crate::orange::clan_point::ClanPoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, Type, query};
use uuid::Uuid;
use crate::orange::Round;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Track {
    id: Uuid,
    self_clan_id: Uuid,
    rival_clan_id: Uuid,
    self_history_point: i64,
    rival_history_point: i64,
    create_time: DateTime<Utc>,
    self_now_point: i64,
    rival_now_point: i64,
    round_id: Uuid,
    result: TrackResult,
}

#[derive(Debug, Clone, PartialEq, Default, Type, Serialize_repr, Deserialize_repr)]
#[repr(i16)]
enum TrackResult {
    Win = 1,
    #[default]
    None = 0,
    Lose = -1,
}

impl Track {
    async fn new(self_clan_point: ClanPoint, rival_clan_point: ClanPoint, conn: &Pool<Postgres>) -> Self {
        let mut track = Self::default();
        let round = Round::select_last(conn).await.unwrap_or_default();
        track.round_id = round.get_id();
        track.self_clan_id = self_clan_point.clan_id;
        track.rival_clan_id = rival_clan_point.clan_id;
        track.create_time = Utc::now();
        track.self_now_point = self_clan_point.point;
        track.rival_now_point = rival_clan_point.point;

        // self < rival
        if track.self_now_point < track.rival_now_point {
            track.self_history_point += 1;
            track.rival_history_point -= 1;
            track.result = TrackResult::Win;
        }

        // self > rival
        if track.self_now_point > track.rival_now_point {
            track.self_history_point -= 1;
            track.rival_history_point += 1;
            track.result = TrackResult::Lose;
        }

        if track.self_now_point == track.rival_now_point {
            track.result = TrackResult::None;
        }
        track
    }

    pub async fn insert(&self, conn: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.track values(DEFAULT, $1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(&self.self_clan_id)
            .bind(&self.rival_clan_id)
            .bind(&self.self_history_point)
            .bind(&self.rival_history_point)
            .bind(now)
            .bind(&self.self_now_point)
            .bind(&self.rival_now_point)
            .bind(&self.round_id)
            .execute(conn)
            .await
    }
}

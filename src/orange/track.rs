use crate::orange::Round;
use crate::orange::clan_point::ClanPoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, Type, query, query_as};
use uuid::Uuid;
use void_log::log_info;

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
    pub async fn new(
        self_clan_point: ClanPoint,
        rival_clan_point: ClanPoint,
        pool: &Pool<Postgres>,
    ) -> Self {
        let round = Round::select_last(pool).await.unwrap_or_default();
        let mut track = Self {
            self_clan_id: self_clan_point.clan_id,
            rival_clan_id: rival_clan_point.clan_id,
            self_history_point: self_clan_point.point,
            rival_history_point: rival_clan_point.point,
            create_time: Utc::now(),
            round_id: round.get_id(),
            ..Default::default()
        };

        if track.self_now_point < track.rival_now_point {
            // self < rival
            track.win()
        } else if track.self_now_point > track.rival_now_point {
            // self > rival
            track.lose()
        } else {
            // Check 10 history
            let self_tracks = Track::select_desc_limit(track.self_clan_id, 10, pool)
                .await
                .unwrap_or_default();
            let rival_tracks = Track::select_desc_limit(track.rival_clan_id, 10, pool)
                .await
                .unwrap_or_default();
            let res10 = check_history(self_tracks, rival_tracks);

            if res10 == TrackResult::None {
                log_info!("{}:{} 拼手速", track.self_clan_id, track.rival_clan_id);
                track.win();
            } else {
                track.result = res10;
            }
        }
        track
    }

    fn win(&mut self) {
        self.self_now_point = self.self_history_point + 1;
        self.rival_now_point = self.rival_history_point - 1;
        self.result = TrackResult::Win;
    }

    fn lose(&mut self) {
        self.self_now_point = self.self_history_point - 1;
        self.rival_now_point = self.rival_history_point + 1;
        self.result = TrackResult::Lose;
    }

    pub async fn select_desc_limit(
        clan_id: Uuid,
        limit: i64,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Self>, Error> {
        query_as::<_, Self>("select * from orange.track where self_clan_id = $1 or rival_clan_id = $1 desc limit $2")
            .bind(clan_id)
            .bind(limit)
            .fetch_all(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
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
            .execute(pool)
            .await
    }
}

/// # History Win Check
fn check_history(self_track: Vec<Track>, rival_track: Vec<Track>) -> TrackResult {
    let self_win = count_win(self_track);
    let rival_win = count_win(rival_track);
    if self_win < rival_win {
        TrackResult::Win
    } else if self_win > rival_win {
        TrackResult::Lose
    } else {
        TrackResult::None
    }
}

/// # Count to history win
fn count_win(tracks: Vec<Track>) -> i64 {
    let mut count = 0;
    for track in tracks {
        if let TrackResult::Win = track.result {
            count += 1
        }
    }
    count
}

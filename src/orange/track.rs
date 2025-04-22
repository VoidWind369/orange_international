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
    pub self_clan_id: Uuid,
    pub rival_clan_id: Uuid,
    self_history_point: i64,
    rival_history_point: i64,
    create_time: DateTime<Utc>,
    pub self_now_point: i64,
    pub rival_now_point: i64,
    round_id: Uuid,
    result: TrackResult,
    round_code: Option<String>,
    self_tag: Option<String>,
    self_name: Option<String>,
    rival_tag: Option<String>,
    rival_name: Option<String>,
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
        pool: &Pool<Postgres>,
        self_clan_point: Option<ClanPoint>,
        rival_clan_point: Option<ClanPoint>,
    ) -> Self {
        let round = Round::select_last(pool).await.unwrap_or_default();

        let scp = self_clan_point.unwrap_or_default();
        let rcp = rival_clan_point.unwrap_or_default();

        let mut track = Self {
            self_clan_id: scp.clan_id,
            rival_clan_id: rcp.clan_id,
            self_history_point: scp.point,
            rival_history_point: rcp.point,
            create_time: Utc::now(),
            round_id: round.get_id(),
            ..Default::default()
        };

        // Track Failed
        if track.self_clan_id == Uuid::default() || track.rival_clan_id == Uuid::default() {
            track.result = TrackResult::None;
            return track;
        }

        // 先手先用奖惩
        if scp.reward_point > 0 {
            // 先登记用奖惩
            track.reward_win(scp, pool).await;
            return track;
        }

        // 对手奖惩
        if rcp.reward_point > 0 {
            // 先登记用奖惩
            track.reward_lose(rcp, pool).await;
            return track;
        }

        if track.self_history_point < track.rival_history_point {
            // self < rival
            track.win(scp, rcp, pool).await;
        } else if track.self_history_point > track.rival_history_point {
            // self > rival
            track.lose(scp, rcp, pool).await;
        } else {
            // Check 10 history
            let self_tracks = Track::select_desc_limit(pool, track.self_clan_id, 10)
                .await
                .unwrap_or_default();
            let rival_tracks = Track::select_desc_limit(pool, track.rival_clan_id, 10)
                .await
                .unwrap_or_default();
            // 按历史10场判断
            track
                .check_history(self_tracks, rival_tracks, scp, rcp, pool)
                .await;
        }
        track
    }

    /// # History Win Check
    async fn check_history(
        &mut self,
        self_track: Vec<Track>,
        rival_track: Vec<Track>,
        scp: ClanPoint,
        rcp: ClanPoint,
        pool: &Pool<Postgres>,
    ) {
        let self_win = count_win(self_track);
        let rival_win = count_win(rival_track);
        if self_win <= rival_win {
            self.win(scp, rcp, pool).await;
        } else {
            self.lose(scp, rcp, pool).await;
        }
    }

    async fn win(&mut self, scp: ClanPoint, rcp: ClanPoint, pool: &Pool<Postgres>) {
        self.self_now_point = self.self_history_point + 1;
        self.rival_now_point = self.rival_history_point - 1;
        self.result = TrackResult::Win;
        scp.update_point(pool, 1).await.unwrap();
        rcp.update_point(pool, -1).await.unwrap();
    }

    async fn lose(&mut self, scp: ClanPoint, rcp: ClanPoint, pool: &Pool<Postgres>) {
        self.self_now_point = self.self_history_point - 1;
        self.rival_now_point = self.rival_history_point + 1;
        self.result = TrackResult::Lose;
        scp.update_point(pool, -1).await.unwrap();
        rcp.update_point(pool, 1).await.unwrap();
    }

    async fn reward_win(&mut self, scp: ClanPoint, pool: &Pool<Postgres>) {
        self.self_now_point = self.self_history_point;
        self.rival_now_point = self.rival_history_point;
        self.result = TrackResult::Win;
        scp.update_point(pool, -1).await.unwrap();
    }

    async fn reward_lose(&mut self, rcp: ClanPoint, pool: &Pool<Postgres>) {
        self.self_now_point = self.self_history_point;
        self.rival_now_point = self.rival_history_point;
        self.result = TrackResult::Lose;
        rcp.update_reward_point(pool, -1).await.unwrap();
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select ot.*, r.code round_code, c1.tag self_tag, c1.name self_name, c2.tag rival_tag, c2.name rival_name from orange.track ot, orange.round r, orange.clan c1, orange.clan c2 where ot.round_id = r.id and self_clan_id = c1.id and ot.rival_clan_id = c2.id order by create_time desc").fetch_all(pool).await
    }

    pub async fn select_registered(
        pool: &Pool<Postgres>,
        self_clan_point: &Option<ClanPoint>,
        rival_clan_point: &Option<ClanPoint>,
    ) -> Result<Self, Error> {
        let sc = if let Some(scp) = self_clan_point {
            scp
        } else {
            return Err(Error::ColumnNotFound("Not Found".to_string()));
        };

        let rc = if let Some(rcp) = rival_clan_point {
            rcp
        } else {
            return Err(Error::ColumnNotFound("Not Found".to_string()));
        };
        log_info!("Track: Self Point{sc:?}");
        log_info!("Track: Rival Point{rc:?}");
        query_as("select ot.*, r.code round_code, c1.tag self_tag, c1.name self_name, c2.tag rival_tag, c2.name rival_name from orange.track ot, orange.round r, orange.clan c1, orange.clan c2 where ot.round_id = r.id and ot.self_clan_id = c1.id and ot.rival_clan_id = c2.id and ((self_clan_id = $1 and rival_clan_id = $2) or (rival_clan_id = $1 and self_clan_id = $2))")
            .bind(sc.clan_id).bind(rc.clan_id).fetch_one(pool).await
    }

    pub async fn select_desc_limit(
        pool: &Pool<Postgres>,
        clan_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Self>, Error> {
        query_as("select ot.*, r.code round_code, c1.tag self_tag, c1.name self_name, c2.tag rival_tag, c2.name rival_name from orange.track ot, orange.round r, orange.clan c1, orange.clan c2 where ot.round_id = r.id and ot.self_clan_id = c1.id and ot.rival_clan_id = c2.id and self_clan_id = $1 or rival_clan_id = $1 order by create_time desc limit $2")
            .bind(clan_id)
            .bind(limit)
            .fetch_all(pool)
            .await
    }

    pub async fn select_round(pool: &Pool<Postgres>, clan_id: Uuid) -> Result<Vec<Self>, Error> {
        let round = Round::select_last(pool).await.unwrap_or_default();
        query_as("select ot.*, r.code round_code, c1.tag self_tag, c1.name self_name, c2.tag rival_tag, c2.name rival_name from orange.track ot, orange.round r, orange.clan c1, orange.clan c2 where ot.round_id = r.id and ot.self_clan_id = c1.id and ot.rival_clan_id = c2.id and (ot.self_clan_id = $1 or ot.rival_clan_id = $1) and ot.round_id = $2 order by ot.create_time desc limit 1")
            .bind(clan_id).bind(round.get_id()).fetch_all(pool).await
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
            .bind(&self.result)
            .execute(pool)
            .await
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

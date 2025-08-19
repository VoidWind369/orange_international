use crate::api::MiddleApi;
use crate::orange::Round;
use crate::orange::clan_point::ClanPoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::postgres::PgQueryResult;
use sqlx::types::Json;
use sqlx::{Error, FromRow, Pool, Postgres, Type, query, query_as};
use uuid::Uuid;
use void_log::log_info;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub self_clan_id: Uuid,
    pub rival_clan_id: Uuid,
    pub self_history_point: i64,
    pub rival_history_point: i64,
    pub create_time: DateTime<Utc>,
    pub self_now_point: i64,
    pub rival_now_point: i64,
    pub round_id: Uuid,
    pub result: TrackResult,
    pub r#type: TrackType,
    pub reward_info: Option<Json<TrackRewardInfo>>,
    pub round_code: Option<String>,
    pub self_tag: Option<String>,
    pub self_name: Option<String>,
    pub rival_tag: Option<String>,
    pub rival_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Type, Serialize_repr, Deserialize_repr)]
#[repr(i16)]
pub enum TrackResult {
    Win = 1,
    #[default]
    None = 0,
    Lose = -1,
}

#[derive(Debug, Clone, PartialEq, Default, Type, Serialize_repr, Deserialize_repr)]
#[repr(i16)]
pub enum TrackType {
    /// # 外部
    External = 0, // 外部
    /// # 内部
    #[default]
    Internal = 1, // 内部
    /// # 友盟
    Alliance = 2, // 友盟
    /// # 奖励局
    Award = 11,
    /// # 处罚局
    Penalty = 12,
}

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct TrackRewardInfo {
    pub self_history: i64,
    pub rival_history: i64,
    pub self_now: i64,
    pub rival_now: i64,
}

impl Track {
    fn set_reward_info(&mut self, reward_info: TrackRewardInfo) {
        self.reward_info = Some(Json(reward_info));
    }
}

impl TrackRewardInfo {
    fn new_history(self_history: i64, rival_history: i64) -> Self {
        Self {
            self_history,
            rival_history,
            ..Default::default()
        }
    }

    /// # 设置扣除奖励券
    fn set_now(&mut self, self_sub: i64, rival_sub: i64) {
        self.self_now = self.self_history + self_sub;
        self.rival_now = self.rival_history + rival_sub
    }
}

fn sql(sql_text: &str) -> String {
    let base_sql = "SELECT
            ot.*,
            r.code round_code,
            c1.tag self_tag,
            c1.NAME self_name,
            c2.tag rival_tag,
            c2.NAME rival_name 
        FROM
            orange.track ot,
            orange.round r,
            orange.clan c1,
            orange.clan c2 
        WHERE
            ot.round_id = r.\"id\" 
            AND ot.self_clan_id = c1.\"id\"
            AND ot.rival_clan_id = c2.\"id\"";
    format!("{base_sql} {sql_text}")
}

impl Track {
    pub async fn new(
        pool: &Pool<Postgres>,
        self_clan_point: Option<ClanPoint>,
        rival_clan_point: Option<ClanPoint>,
        self_tag: &str,
        is_global: bool,
    ) -> Self {
        let round = Round::select_last(pool).await.unwrap_or_default();

        // 初始化积分
        let scp = self_clan_point.unwrap_or_default();
        let rcp = rival_clan_point.unwrap_or_default();

        // 初始化奖励券
        let mut reward_info = TrackRewardInfo::new_history(scp.reward_point, rcp.reward_point);

        // 初始化Track
        let mut track = Self {
            self_clan_id: scp.clan_id,
            rival_clan_id: rcp.clan_id,
            self_history_point: scp.point,
            rival_history_point: rcp.point,
            create_time: Utc::now(),
            round_id: round.get_id(),
            ..Default::default()
        };

        // ****************Track Failed 调用中间库****************
        if track.self_clan_id == Uuid::default() || track.rival_clan_id == Uuid::default() {
            let ma = MiddleApi::new(self_tag, is_global).await.unwrap();
            return ma.check_win(pool, track, is_global, self_tag).await;
        }
        // ****************Track Failed 调用中间库****************

        // ***********************奖惩阶段***********************
        // 先手先用奖惩
        if scp.reward_point > 0 {
            // 先登记用奖惩
            reward_info.set_now(1, 0);
            track.set_reward_info(reward_info);
            track
                .reward(scp, pool, true, TrackResult::Win)
                .await;
            return track;
        }
        if rcp.reward_point < 0 {
            reward_info.set_now(0, 1);
            track.set_reward_info(reward_info);
            track
                .reward(rcp, pool, false, TrackResult::Win)
                .await;
            return track;
        }

        // 对手奖惩
        if rcp.reward_point > 0 {
            // 先登记用奖惩
            reward_info.set_now(0, 1);
            track.set_reward_info(reward_info);
            track
                .reward(rcp, pool, true, TrackResult::Lose)
                .await;
            return track;
        }
        if rcp.reward_point < 0 {
            // 先登记用奖惩
            reward_info.set_now(1, 0);
            track.set_reward_info(reward_info);
            track
                .reward(scp, pool, false, TrackResult::Lose)
                .await;
            return track;
        }
        // ***********************奖惩阶段***********************

        // ***********************积分阶段***********************
        // 无奖励写入
        track.set_reward_info(reward_info);
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
        // ***********************积分阶段***********************

        log_info!("{:?}", &track);
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
        let self_win = count_win(self_track, scp.clan_id);
        let rival_win = count_win(rival_track, rcp.clan_id);
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

    /// # reward公共方法
    /// is_reward_point: true传本方
    async fn reward(
        &mut self,
        cp: ClanPoint,
        pool: &Pool<Postgres>,
        is_reward_point: bool,
        track_result: TrackResult,
    ) {
        self.self_now_point = self.self_history_point;
        self.rival_now_point = self.rival_history_point;
        self.result = track_result;
        if is_reward_point {
            self.r#type = TrackType::Award;
            cp.update_reward_point_base(pool, -1).await.unwrap();
        } else {
            self.r#type = TrackType::Penalty;
            cp.update_reward_point_base(pool, 1).await.unwrap();
        }
    }

    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as(&sql("order by create_time desc"))
            .fetch_all(pool)
            .await
    }

    pub async fn select_registered(
        pool: &Pool<Postgres>,
        self_clan_point: &Option<ClanPoint>,
        round: &Round,
    ) -> Result<Self, Error> {
        let sc = if let Some(scp) = self_clan_point {
            scp
        } else {
            return Err(Error::ColumnNotFound("Not Found".to_string()));
        };
        log_info!("Track: Self Point{sc:?}");
        query_as(&sql(
            "and (self_clan_id = $1 or rival_clan_id = $1) and round_id = $2",
        ))
            .bind(sc.clan_id)
            .bind(round.get_id())
            .fetch_one(pool)
            .await
    }

    pub async fn select_desc_limit(
        pool: &Pool<Postgres>,
        clan_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Self>, Error> {
        query_as(&sql(
            "and (self_clan_id = $1 or rival_clan_id = $1) order by create_time desc limit $2",
        ))
            .bind(clan_id)
            .bind(limit)
            .fetch_all(pool)
            .await
    }

    pub async fn select_round(pool: &Pool<Postgres>, clan_id: Uuid) -> Result<Vec<Self>, Error> {
        let round = Round::select_last(pool).await.unwrap_or_default();
        Self::select_clan_round(pool, clan_id, round.get_id()).await
    }

    ///
    pub async fn select_clan_round(
        pool: &Pool<Postgres>,
        clan_id: Uuid,
        round_id: Uuid,
    ) -> Result<Vec<Self>, Error> {
        query_as(&sql("and (ot.self_clan_id = $1 or ot.rival_clan_id = $1) and ot.round_id = $2 order by ot.create_time desc limit 1"))
            .bind(clan_id).bind(round_id).fetch_all(pool).await
    }

    pub async fn select(pool: &Pool<Postgres>, id: Uuid) -> Result<Self, Error> {
        query_as(&(sql("and id = $1")))
            .bind(id)
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.track values(DEFAULT, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(&self.self_clan_id)
            .bind(&self.rival_clan_id)
            .bind(&self.self_history_point)
            .bind(&self.rival_history_point)
            .bind(now)
            .bind(&self.self_now_point)
            .bind(&self.rival_now_point)
            .bind(&self.round_id)
            .bind(&self.result)
            .bind(&self.r#type)
            .bind(&self.reward_info)
            .execute(pool)
            .await
    }

    /// # 解除本场登记
    pub async fn delete(pool: &Pool<Postgres>, id: Uuid) -> Result<PgQueryResult, Error> {
        let round = Round::select_last(pool).await.unwrap_or_default();
        let track = Self::select(pool, id).await?;
        if track.round_id == round.get_id() {
            query("delete from orange.track where id = $1")
                .bind(id)
                .execute(pool)
                .await
        } else {
            Err(Error::RowNotFound)
        }
    }
}

/// # Count to history win
fn count_win(tracks: Vec<Track>, clan_id: Uuid) -> i64 {
    let mut count = 0;
    for track in tracks {
        let self_win = TrackResult::Win == track.result && clan_id == track.self_clan_id;
        let rival_win = TrackResult::Lose == track.result && clan_id == track.rival_clan_id;
        if self_win || rival_win {
            count += 1
        }
    }
    count
}

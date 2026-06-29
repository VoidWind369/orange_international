use axum::http::status;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, postgres::PgQueryResult};
use uuid::Uuid;
use void_log::log_info;

use crate::{
    api::{BZ_UUID, MiddleTrackApi, War},
    orange::{Clan, ClanPoint, ClanStatus, Round, Track, TrackResult, TrackType},
};

pub enum RegResponse {
    Successful,
    NotRound,
    TagNotFound,
    ClanNotFound(ClanTag),
    ClanNotOwned(ClanTag),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackReg {
    self_tag: String,
    rival_tag: Option<String>,
    last: Option<bool>,
    is_global: Option<bool>,
}

pub struct ClanTag {
    pub tag: String,
    pub is_global: bool,
}

impl TrackReg {
    /// # 是否后手
    /// * 默认先手
    fn is_last(&self) -> bool {
        self.last.unwrap_or_default()
    }

    /// # 是否国际服
    /// * 默认国际服
    fn is_global(&self) -> bool {
        self.is_global.unwrap_or(true)
    }

    fn get_self_tag(&self) -> ClanTag {
        ClanTag {
            tag: self.self_tag.to_ascii_uppercase(),
            is_global: self.is_global(),
        }
    }

    fn get_rival_tag(&self) -> Option<String> {
        self.rival_tag.clone()
    }

    /// # 登记
    pub async fn new_reg(&self, pool: &Pool<Postgres>) -> RegResponse {
        // 检查登记时间
        let round = Round::select_last(pool).await.unwrap_or_default();
        if round.check_not_now().await {
            return RegResponse::NotRound;
        };

        // 获取本家tag
        let self_tag = self.get_self_tag();

        // 获取对家tag
        let rival_tag = if let Some(tag) = self.get_rival_tag() {
            // 手动登记
            ClanTag::new(tag, self.is_global())
        } else {
            // 自动登记
            if let Some(tag) = self_tag.get_war_rival_tag().await {
                ClanTag::new(tag, self.is_global())
            } else {
                return RegResponse::TagNotFound;
            }
        };

        // 本家数据库Clan
        let self_clan = if let Ok(clan) = self_tag.get_clan(pool).await {
            clan
        } else {
            return RegResponse::ClanNotFound(self_tag);
        };

        // 判断本方状态
        let self_clan = if self_clan.status.is_some_and(|t| t == ClanStatus::Ready) {
            self_clan
        } else {
            return RegResponse::ClanNotOwned(self_tag);
        };

        // 对家数据库Clan
        if let Ok(rival_clan) = rival_tag.get_clan(pool).await {
            // 判断对方状态
            match rival_clan.status {
                // 正常部落
                Some(ClanStatus::Ready) => {
                    let mut self_point = self_clan.point_select(pool).await.unwrap_or_default();
                    let mut rival_point = rival_clan.point_select(pool).await.unwrap_or_default();
                    self_point.clan_id = self_clan.get_id();
                    rival_point.clan_id = rival_clan.get_id();
                    self.ready_track(&self_point, &rival_point, pool).await;
                }
                // 黑名单部落
                Some(ClanStatus::Blacklist) => self.not_own_track(&ClanStatus::Blacklist),
                // 友盟部落
                Some(status) => {
                    if let Ok(api) = self_tag.middle_api().await {
                        self.ally_track(&api, pool).await;
                    } else {
                        self.not_own_track(&status)
                    };
                }
                // 系统故障
                None => return RegResponse::ClanNotOwned(rival_tag),
            }
        } else {
            rival_tag.middle_api().await;
        };

        RegResponse::Successful
    }

    async fn ready_track(
        &self,
        self_point: &ClanPoint,
        rival_point: &ClanPoint,
        pool: &Pool<Postgres>,
    ) -> (TrackResult, TrackType) {
        // 先后手转换
        let (first, last) = if self.is_last() {
            (rival_point, self_point)
        } else {
            (self_point, rival_point)
        };

        if first.reward_point > 0 {
            // 惩罚先手输
            (TrackResult::Lose, TrackType::Penalty)
        } else if first.reward_point < 0 {
            // 奖励先手赢
            (TrackResult::Lose, TrackType::Award)
        } else if last.reward_point > 0 {
            // 惩罚后手输
            (TrackResult::Lose, TrackType::Penalty)
        } else if last.reward_point < 0 {
            // 奖励对手赢
            (TrackResult::Lose, TrackType::Award)
        } else {
            // 正常对局
            (point_check(first, last, pool).await, TrackType::Internal)
        }
    }

    async fn ally_track(
        &self,
        api: &MiddleTrackApi,
        pool: &Pool<Postgres>,
    ) -> (TrackResult, TrackType) {
        // 合作联盟
        let self_tag = format!("#{}", api.my_tag.replace("#", ""));
        let rival_tag = format!("#{}", api.opp_tag.replace("#", ""));
        // 格式化输赢tag
        let win_tag = format!("#{}", api.win_tag.replace("#", ""));

        let (status, series_id) = if api.err {
            (ClanStatus::Other, None)
        } else {
            (ClanStatus::Ally, Some(Uuid::parse_str(BZ_UUID).unwrap()))
        };

        let mut clan = Clan {
            tag: Some(rival_tag.clone()),
            name: api.opp_name.clone(),
            status: Some(status),
            series_id,
            ..Default::default()
        };

        // 查询对家在数据库记录,有就更新,没有就新增
        if let Ok(rc) = ClanTag::new(rival_tag.clone(), self.is_global())
            .get_clan(pool)
            .await
        {
            log_info!("有缓存: {}", rc.name.clone().unwrap());
            if rc.status.is_some_and(|x| x != status) {
                clan.id = rc.id;
                clan.update(pool).await.unwrap();
            }
        } else {
            let insert_res = clan.insert(pool).await.unwrap();
            log_info!("新增外部: {}", insert_res.rows_affected());
        };

        // 判断输赢写入Track
        if win_tag.eq(&rival_tag) {
            (TrackResult::Lose, TrackType::Alliance)
        } else if api.err {
            (TrackResult::None, TrackType::External)
        } else {
            (TrackResult::Win, TrackType::Alliance)
        }
    }

    fn not_own_track(&self, status: &ClanStatus) {
        // 必赢
    }
}

// 常规积分比对
async fn point_check(
    first_point: &ClanPoint,
    last_point: &ClanPoint,
    pool: &Pool<Postgres>,
) -> TrackResult {
    if first_point.point < last_point.point {
        // 先手赢
        TrackResult::Win
    } else if first_point.point > last_point.point {
        // 后手赢
        TrackResult::Lose
    } else {
        // 按历史10场判断
        let first_tracks = Track::select_desc_limit(pool, first_point.clan_id, 10)
            .await
            .unwrap_or_default();
        let last_tracks = Track::select_desc_limit(pool, last_point.clan_id, 10)
            .await
            .unwrap_or_default();
        // 计数
        if count_win(first_tracks, first_point.clan_id)
            <= count_win(last_tracks, last_point.clan_id)
        {
            // 先手赢
            TrackResult::Win
        } else {
            // 后手赢
            TrackResult::Lose
        }
    }
}

// 对比历史战绩
fn count_win(tracks: Vec<Track>, clan_id: Uuid) -> i64 {
    let mut count = 0;
    for track in tracks {
        let first_win = TrackResult::Win == track.result && clan_id == track.self_clan_id;
        let last_win = TrackResult::Lose == track.result && clan_id == track.rival_clan_id;
        if first_win || last_win {
            count += 1
        }
    }
    count
}

impl ClanTag {
    fn new(tag: String, is_global: bool) -> Self {
        Self { tag, is_global }
    }

    async fn get_clan(&self, pool: &Pool<Postgres>) -> Result<Clan, sqlx::Error> {
        Clan::select_tag(pool, &self.tag, self.is_global).await
    }

    async fn get_war_rival_tag(&self) -> Option<String> {
        let war = War::get(&self.tag).await;
        if let Some(opponent_clan_tag) = war.opponent.unwrap_or_default().tag {
            Some(opponent_clan_tag)
        } else {
            // 未开战
            None
        }
    }
}

pub async fn reverse(pool: &Pool<Postgres>, track_id: Uuid) -> sqlx::Result<PgQueryResult> {
    let track = Track::select(pool, track_id).await;
    if let Ok(track) = track {
        // 非内部匹配禁止逆转
        if track.r#type != TrackType::Internal {
            return Err(sqlx::Error::PoolClosed);
        };

        // 查先手积分
        let self_point = ClanPoint::select(pool, track.self_clan_id).await?;

        // 查对手积分
        let rival_point = ClanPoint::select(pool, track.rival_clan_id).await?;

        match track.result {
            TrackResult::Win => {
                let mut re_track = track;
                re_track.result = TrackResult::Lose;
                re_track.self_now_point = re_track.self_history_point - 1;
                re_track.rival_now_point = re_track.rival_history_point + 1;
                re_track.r#type = TrackType::Reverse;

                // 更新积分
                self_point.update_point(pool, -2).await?;
                rival_point.update_point(pool, 2).await?;

                re_track.insert(pool).await
            }
            TrackResult::None => return Err(sqlx::Error::PoolClosed),
            TrackResult::Lose => {
                let mut re_track = track;
                re_track.result = TrackResult::Lose;
                re_track.self_now_point = re_track.self_history_point + 1;
                re_track.rival_now_point = re_track.rival_history_point - 1;
                re_track.r#type = TrackType::Reverse;

                // 更新积分
                self_point.update_point(pool, 2).await.unwrap();
                rival_point.update_point(pool, -2).await.unwrap();

                re_track.insert(pool).await
            }
        }
    } else {
        Err(sqlx::Error::PoolClosed)
    }
}

use chrono::Utc;
use sqlx::{Pool, Postgres, types::Json};
use uuid::Uuid;
use void_log::{log_info, log_warn};

use crate::{
    api::{BZ_UUID, MiddleTrackApi, War},
    core::{ClanTag, RegResponse, TrackReg},
    orange::{Clan, ClanPoint, ClanStatus, Round, Track, TrackResult, TrackRewardInfo, TrackType},
};

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

        let mut track = Track {
            self_clan_id: self_clan.get_id(),
            create_time: Utc::now(),
            round_id: round.get_id(),
            reward_info: todo!(),
            round_code: Some(round.get_code()),
            self_tag: self_clan.tag,
            self_name: self_clan.name,
            ..Default::default()
        };

        // 对家数据库Clan
        if let Ok(rival_clan) = rival_tag.get_clan(pool).await {
            // 先统计积分
            let mut self_point = self_clan.point_select(pool).await.unwrap_or_default();
            let mut rival_point = rival_clan.point_select(pool).await.unwrap_or_default();
            self_point.clan_id = self_clan.get_id(); // 可能无记录先赋值id
            rival_point.clan_id = rival_clan.get_id(); // 可能无记录先赋值id

            // 补全对手信息
            track.rival_clan_id = rival_clan.get_id();
            track.rival_tag = rival_clan.tag;
            track.rival_name = rival_clan.name;

            // 初始化积分信息
            track.self_history_point = self_point.point;
            track.rival_history_point = rival_point.point;
            track.self_now_point = self_point.point; // 先默认无变化
            track.rival_now_point = rival_point.point; // 先默认无变化

            // 判断对方状态
            match rival_clan.status {
                // 正常部落
                Some(ClanStatus::Ready) => {
                    self.ready_track(&mut self_point, &mut rival_point, track, pool)
                        .await
                }
                // 黑名单部落
                Some(ClanStatus::Blacklist) => self.not_own_track(&ClanStatus::Blacklist, track),
                // 友盟部落/外部/锁定
                Some(status) => {
                    if let Ok(api) = self_tag.middle_api().await {
                        self.ally_track(&api, track, pool).await
                    } else {
                        self.not_own_track(&status, track)
                    }
                }
                // 系统故障
                None => return RegResponse::ClanNotOwned(rival_tag),
            };
        } else {
            // 数据库无数据访问中间库
            if let Ok(api) = self_tag.middle_api().await {
                self.ally_track(&api, track, pool).await
            } else {
                log_warn!("数据严重缺失");
                self.not_own_track(&ClanStatus::Other, track)
            };
        };

        // 数据入库
        track.insert(pool).await.unwrap();

        RegResponse::Successful(track)
    }

    /// # 正常与奖惩登记
    async fn ready_track(
        &self,
        self_point: &mut ClanPoint,
        rival_point: &mut ClanPoint,
        mut track: Track,
        pool: &Pool<Postgres>,
    ) -> Track {
        // 先后手转换
        let (first, last) = if self.is_last() {
            (rival_point, self_point)
        } else {
            (self_point, rival_point)
        };

        // 转化后track的self=first，rival=last
        let mut reward_info = TrackRewardInfo {
            self_history: first.reward_point,
            rival_history: last.reward_point,
            self_now: first.reward_point, // 先默认无变化
            rival_now: last.reward_point, // 先默认无变化
        };

        // 直接赋值输赢、类别
        (track.result, track.r#type) = if first.reward_point > 0 {
            // 奖励先手赢
            first.update_reward_point_base(pool, -1).await.unwrap();
            track.result = TrackResult::Win;
            track.r#type = TrackType::Award;
            reward_info.self_now = first.reward_point - 1;
            (TrackResult::Lose, TrackType::Award)
        } else if first.reward_point < 0 {
            // 惩罚先手输
            first.update_reward_point_base(pool, 1).await.unwrap();
            track.result = TrackResult::Lose;
            track.r#type = TrackType::Award;
            reward_info.self_now = first.reward_point + 1;
            (TrackResult::Lose, TrackType::Penalty)
        } else if last.reward_point > 0 {
            // 奖励后手赢
            last.update_reward_point_base(pool, -1).await.unwrap();
            track.result = TrackResult::Lose;
            track.r#type = TrackType::Award;
            reward_info.self_now = last.reward_point - 1;
            (TrackResult::Lose, TrackType::Award)
        } else if last.reward_point < 0 {
            // 惩罚后手输
            last.update_reward_point_base(pool, 1).await.unwrap();
            track.result = TrackResult::Win;
            track.r#type = TrackType::Penalty;
            reward_info.self_now = last.reward_point + 1;
            (TrackResult::Lose, TrackType::Penalty)
        } else {
            // 正常对局
            (
                point_check(first, last, &mut track, pool).await,
                TrackType::Internal,
            )
        };

        track.reward_info = Some(Json(reward_info));

        track
    }

    /// # 友盟登记
    async fn ally_track(
        &self,
        api: &MiddleTrackApi,
        mut track: Track,
        pool: &Pool<Postgres>,
    ) -> Track {
        // 合作联盟
        let rival_tag = format!("#{}", api.opp_tag.replace("#", ""));
        track.rival_tag = Some(rival_tag.clone());
        track.rival_name = api.opp_name.clone();
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
            track.result = TrackResult::Lose;
            track.r#type = TrackType::Alliance;
        } else if api.err {
            track.result = TrackResult::None;
            track.r#type = TrackType::External;
        } else {
            track.result = TrackResult::Win;
            track.r#type = TrackType::Alliance;
        };
        track
    }

    fn not_own_track(&self, status: &ClanStatus, mut track: Track) -> Track {
        // 必赢
        track.r#type = match status {
            ClanStatus::Locked | ClanStatus::Other => TrackType::External,
            ClanStatus::Blacklist => TrackType::Blacklist,
            ClanStatus::Ready => TrackType::Internal,
            ClanStatus::Ally => TrackType::Alliance,
        };
        track.result = TrackResult::None;
        track
    }
}

// 常规积分比对
async fn point_check(
    first_point: &mut ClanPoint,
    last_point: &mut ClanPoint,
    track: &mut Track,
    pool: &Pool<Postgres>,
) -> TrackResult {
    let res = if first_point.point < last_point.point {
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
        // 历史战绩计数
        if count_win(first_tracks, first_point.clan_id)
            <= count_win(last_tracks, last_point.clan_id)
        {
            // 先手赢
            TrackResult::Win
        } else {
            // 后手赢
            TrackResult::Lose
        }
    };
    match res {
        TrackResult::Win => {
            first_point.point += 1;
            last_point.point -= 1;
            track.self_now_point = first_point.point;
            track.rival_now_point = last_point.point;
            first_point.insert_or_update(pool).await.unwrap();
            last_point.insert_or_update(pool).await.unwrap();
        }
        TrackResult::None => todo!(),
        TrackResult::Lose => {
            first_point.point -= 1;
            last_point.point += 1;
            track.self_now_point = first_point.point;
            track.rival_now_point = last_point.point;
            first_point.insert_or_update(pool).await.unwrap();
            last_point.insert_or_update(pool).await.unwrap();
        }
    };
    res
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

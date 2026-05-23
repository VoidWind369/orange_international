use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, postgres::PgQueryResult};
use uuid::Uuid;

use crate::{
    api::War,
    orange::{Clan, ClanPoint, Round, Track, TrackResult, TrackType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackReg {
    self_tag: String,
    rival_tag: Option<String>,
    last: Option<bool>,
    is_global: Option<bool>,
}

struct ClanTag {
    tag: String,
    is_global: bool,
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
    pub async fn new_reg(&self, pool: &Pool<Postgres>) -> Option<i16> {
        // 检查登记时间
        let round = Round::select_last(pool).await.unwrap_or_default();
        if round.check_not_now().await {
            return None;
        };

        let self_tag = self.get_self_tag();

        let rival_tag = if let Some(tag) = self.get_rival_tag() {
            ClanTag::new(tag, self.is_global())
        } else {
            if let Some(tag) = self_tag.get_war_rival_tag().await {
                ClanTag::new(tag, self.is_global())
            } else {
                return None;
            }
        };

        let self_clan = if let Ok(clan) = self_tag.get_clan(pool).await {
            clan
        } else {
            return None;
        };

        let rival_clan = if let Ok(clan) = rival_tag.get_clan(pool).await {
            clan
        } else {
            return None;
        };

        let (first_clan, last_clan) = if self.is_last() {
            (rival_clan, self_clan)
        } else {
            (self_clan, rival_clan)
        };
        Some(1)
    }
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

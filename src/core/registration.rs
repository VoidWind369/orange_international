use sqlx::{Pool, Postgres, postgres::PgQueryResult};
use uuid::Uuid;

use crate::orange::{ClanPoint, Track, TrackResult, TrackType};

pub fn new_reg(pool: &Pool<Postgres>, self_tag: &str, rival_tag: &str) {
    todo!()
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

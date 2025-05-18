mod clan;
mod clan_point;
mod operate_log;
mod round;
mod series;
mod track;

use crate::api::War;
use crate::orange::clan_point::ClanPoint;
use crate::orange::operate_log::OperateLog;
use crate::system::{User, UserInfo};
use crate::{AppState, api};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use axum_auth::AuthBearer;
pub use clan::Clan;
pub use clan::ClanUser;
pub use round::Round;
use serde_json::Value;
use sqlx::Error;
use sqlx::postgres::PgQueryResult;
pub use track::*;
use uuid::Uuid;
use void_log::{log_info, log_warn};

pub fn router() -> Router<AppState> {
    Router::new()
        // 部落相关
        .route("/clan", get(clans).post(clan_insert).put(clan_update))
        .route("/clan_search", post(clan_search))
        .route("/clan/{id}", get(clan).delete(clan_delete))
        .route("/clan/{tag}/{is_global}", get(clan_tag))
        .route("/clan_info/{tag}", get(clan_info))
        // 部落积分相关
        .route("/clan_point", put(clan_reward_point))
        .route("/clan_point/{id}", get(clan_point))
        // 时间发布相关
        .route("/round", get(rounds).post(round_insert))
        .route("/last_round", get(last_round))
        // 对战记录相关
        .route("/track", get(tracks).post(new_track))
        .route("/track/{id}", get(track_round))
        // 用户关联相关
        .route("/user_clans", get(user_clans))
        .route("/user_clans/{id}", get(userid_clans))
        .route("/clan_user", post(insert_cu).delete(delete_cu))
        // 操作日志相关
        .route("/operate_log", get(operate_logs))
}

/// # All Clan
async fn clans(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    if !token.eq("cfa*clan*select") {
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Clan::select_all(&app_state.pool).await.unwrap();

    // let res = Clan::select_all(&app_state.pool).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn clan(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Clan::select(&app_state.pool, id).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn clan_tag(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Path((tag, is_global)): Path<(String, bool)>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if !token.eq("cfa*clan*select") {
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************
    log_info!("Clan {} {}", &tag, is_global);
    let tag = format!("#{tag}").to_uppercase();

    if let Ok(clan) = Clan::select_tag(&app_state.pool, &tag, 1, is_global).await {
        log_info!("{:?}", clan);
        (StatusCode::OK, Json(clan))
    } else {
        (StatusCode::NOT_FOUND, Json::default())
    }
}

async fn clan_info(AuthBearer(token): AuthBearer, Path(tag): Path<String>) -> impl IntoResponse {
    // ********************鉴权********************
    if !token.eq("cfa*clan*select") {
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************
    let tag = format!("#{tag}").to_uppercase();
    let res = api::Clan::get(&tag).await.info();
    (StatusCode::OK, Json(res))
}

async fn clan_search(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(text): Json<String>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if !token.eq("cfa*clan*select") {
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    log_info!("Clan {}", &text);
    if let Ok(clan) = Clan::select_search(&app_state.pool, &text).await {
        log_info!("{:?}", clan);
        (StatusCode::OK, Json(clan))
    } else {
        (StatusCode::NOT_FOUND, Json::default())
    }
}

async fn clan_insert(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    headers: HeaderMap,
    Json(data): Json<Clan>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    // 是否自动
    let res = if headers.get("Auto").is_some() {
        data.api_insert(&app_state.pool).await
    } else {
        data.insert(&app_state.pool).await
    };

    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

async fn clan_update(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    headers: HeaderMap,
    Json(data): Json<Clan>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    log_info!("{:?}", data);
    let res = if data.status.is_some() {
        log_info!("update status");
        data.update_status(&app_state.pool).await
    } else if headers.get("Auto").is_some() {
        log_info!("auto update clan");
        // 是否自动
        data.api_insert(&app_state.pool).await
    } else {
        log_info!("update clan");
        data.update(&app_state.pool).await
    };

    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

async fn clan_delete(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Clan::delete(&app_state.pool, id).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

async fn rounds(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Round::select_all(&app_state.pool).await.unwrap();
    (StatusCode::OK, Json(res))
}

async fn last_round(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    log_info!("User Token {}", token);
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Round::select_last(&app_state.pool).await.unwrap();
    (StatusCode::OK, Json(res))
}

async fn round_insert(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    // ********************鉴权********************
    log_info!("User Token {}", token);
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    if let Some(time_str) = data["time"].as_str() {
        let res = Round::insert(time_str, &app_state.pool).await;
        let rows_affected = res.unwrap_or_default().rows_affected();
        (StatusCode::OK, Json(rows_affected as i64))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(-2))
    }
}

async fn tracks(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Track::select_all(&app_state.pool).await.unwrap();
    (StatusCode::OK, Json(res))
}

async fn track_round(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Track::select_desc_limit(&app_state.pool, id, 20)
        .await
        .unwrap();
    (StatusCode::OK, Json(res))
}

///
///
/// # New Track
///
/// * `State(app_state)`:
/// * `Json(data)`:
///
/// returns: impl IntoResponse+Sized
///
/// # Examples
///
/// ```
/// {
///   “self_tag”: "#qwer0987",
///   "rival_tag": "#asdf1234",
///   "is_intel": true,
/// }
/// ```
async fn new_track(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    // ********************鉴权********************
    log_info!("User Token {}", token);
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    // 检查登记时间
    let round = Round::select_last(&app_state.pool)
        .await
        .unwrap_or_default();
    if round.check_not_now().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json::default());
    };

    // 默认国际服
    let is_global = if let Some(global) = data.get("is_global") {
        global.as_bool().unwrap_or(true)
    } else {
        true
    };

    // 获取先后手
    let last = if let Some(l) = data.get("last") {
        l.as_bool().unwrap_or_default()
    } else {
        false
    };

    // 获取本家标签
    let self_tag = data["self_tag"].as_str().unwrap_or_default().to_string();

    // 获取对家标签
    let rival_tag = if let Some(tag) = data.get("rival_tag") {
        log_info!("手动登记");
        tag.as_str().unwrap_or_default().to_string()
    } else if is_global {
        log_info!("国际服自动登记");
        // 查对面标签
        let war = War::get(&self_tag).await;
        if let Some(opponent_clan_tag) = war.opponent.unwrap().tag {
            opponent_clan_tag
        } else {
            // 未开战
            log_warn!("未开战");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json::default());
        }
    } else {
        log_info!("国服无接口");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json::default());
    };

    let (self_tag, rival_tag) = if last {
        (rival_tag, self_tag)
    } else {
        (self_tag, rival_tag)
    };

    log_info!("登记1: 获取双方标签 {self_tag} vs {rival_tag}");

    // 查询本家加盟状态
    let self_clan = Clan::select_tag(&app_state.pool, &self_tag, 1, is_global).await;

    // 查询对家加盟状态
    let rival_clan = Clan::select_tag(&app_state.pool, &rival_tag, 1, is_global).await;

    // 本家积分数据
    let (self_point, has_self_tracks) = if let Ok(ref clan) = self_clan {
        log_info!("登记2: 本家加盟状态 {:?}", &clan);
        let mut point = clan.point_select(&app_state.pool).await.unwrap_or_default();
        log_info!("登记3: 本家积分状态 {:?}", &point);
        point.clan_id = clan.id.unwrap_or_default();

        let cst = Track::select_round(&app_state.pool, point.clan_id)
            .await
            .unwrap();

        (Some(point), !cst.is_empty())
    } else {
        log_info!("标签错误");
        (None, false)
    };

    // 对家积分数据
    let (rival_point, has_rival_tracks) = if let Ok(ref clan) = rival_clan {
        log_info!("登记2: 对家加盟状态 {:?}", &clan);
        let mut point = clan.point_select(&app_state.pool).await.unwrap_or_default();
        log_info!("登记3: 对家积分状态 {:?}", &point);
        point.clan_id = clan.id.unwrap_or_default();

        let crt = Track::select_round(&app_state.pool, point.clan_id)
            .await
            .unwrap();

        (Some(point), !crt.is_empty())
    } else {
        log_info!("盟外部落");
        (None, false)
    };

    // 本轮已登记直接返回
    if let Ok(track) = Track::select_registered(&app_state.pool, &self_point, &round).await {
        log_info!("已登记 {:?}", track);
        return (StatusCode::OK, Json(track));
    };

    // 预查限制重复登记
    if has_self_tracks || has_rival_tracks {
        log_warn!("预查重复登记");
        return (StatusCode::FORBIDDEN, Json::default());
    }

    // 添加Track获取输赢（本盟/中间库）
    let track = Track::new(
        &app_state.pool,
        self_point,
        rival_point,
        &self_tag,
        is_global,
    )
    .await;

    // 添加track记录（数据库Unique限制重复）
    let track_res = if let Ok(qr) = track.insert(&app_state.pool).await {
        qr.rows_affected()
    } else {
        log_warn!("数据库Unique重复");
        return (StatusCode::FORBIDDEN, Json(track));
    };

    // 更新self
    let self_point = if self_clan.is_ok() {
        ClanPoint::new(track.self_clan_id, track.self_now_point)
            .insert_or_update(&app_state.pool)
            .await
            .unwrap()
            .rows_affected()
    } else {
        0
    };

    // 更新rival
    let rival_point = if rival_clan.is_ok() {
        ClanPoint::new(track.rival_clan_id, track.rival_now_point)
            .insert_or_update(&app_state.pool)
            .await
            .unwrap()
            .rows_affected()
    } else {
        0
    };

    log_info!(
        "self: {} | rival: {} | track: {track_res}",
        self_point,
        rival_point
    );

    log_info!("新登记 {:?}", track);
    (StatusCode::OK, Json(track))
}

async fn user_clans(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let user_info = UserInfo::get_user(&token).await.unwrap_or_default();
    let user = User::select(&app_state.pool, user_info.get_id())
        .await
        .unwrap_or_default();
    let clans = user.user_clans(&app_state.pool).await.unwrap_or_default();
    (StatusCode::OK, Json(clans))
}

async fn userid_clans(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let user = User::select(&app_state.pool, id).await.unwrap_or_default();
    let clans = user.user_clans(&app_state.pool).await.unwrap_or_default();
    (StatusCode::OK, Json(clans))
}

async fn insert_cu(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<ClanUser>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    if let Ok(r) = data.insert(&app_state.pool).await {
        (StatusCode::OK, Json(r.rows_affected()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json::default())
    }
}

async fn delete_cu(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<ClanUser>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    if let Ok(r) = data.delete(&app_state.pool).await {
        (StatusCode::OK, Json(r.rows_affected()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json::default())
    }
}

async fn clan_point(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = ClanPoint::select(&app_state.pool, id).await;
    log_info!("clan_point {id}: {res:?}");
    if let Ok(r) = res {
        (StatusCode::OK, Json(r))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json::default())
    }
}

async fn clan_reward_point(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<OperateLog>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************
    
    let res = data.new_reward(&app_state.pool).await;
    if let Ok(r) = res {
        (StatusCode::OK, Json(r.rows_affected()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json::default())
    }
}

async fn operate_logs(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    if !token.eq("cfa*operate*log*select") {
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = OperateLog::select_all(&app_state.pool).await;
    if let Ok(r) = res {
        (StatusCode::OK, Json(r))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json::default())
    }
}

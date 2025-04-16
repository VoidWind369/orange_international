mod clan;
mod clan_point;
mod round;
mod series;
mod track;

use crate::api::War;
use crate::orange::clan_point::ClanPoint;
use crate::system::UserInfo;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, head};
use axum::{Json, Router};
use axum_auth::AuthBearer;
pub use clan::Clan;
pub use round::Round;
use serde_json::Value;
pub use track::Track;
use uuid::Uuid;
use void_log::{log_info, log_warn};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/clan", get(clans).post(clan_insert))
        .route("/clan/{id}", get(clan))
        .route("/round", get(rounds).post(round_insert))
        .route("/last_round", get(last_round))
        .route("/track", get(tracks).post(new_track))
}

/// # All Clan
async fn clans(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = Clan::select_all(&app_state.pool).await.unwrap();
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
    (StatusCode::OK, Json(rows_affected as i64))
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

    let self_tag = data["self_tag"].as_str().unwrap_or_default();

    // 获取对家标签
    let rival_tag = if let Some(tag) = data.get("rival_tag") {
        tag.as_str().unwrap_or_default()
    } else {
        let war = War::get(self_tag).await;
        if let Some(opponent_clan) = war.opponent {
            &opponent_clan.tag.unwrap_or_default()
        } else {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json::default());
        }
    };

    // 默认国际服
    let is_intel = if let Some(intel) = data.get("is_intel") {
        intel.as_bool().unwrap_or(true)
    } else {
        true
    };

    // Search Clan
    let self_clan = Clan::select_tag(self_tag, is_intel, &app_state.pool)
        .await
        .unwrap_or_default();
    let rival_clan = Clan::select_tag(rival_tag, is_intel, &app_state.pool)
        .await
        .unwrap_or_default();

    log_info!("Self {:?}", &self_clan);
    log_info!("Rival {:?}", &rival_clan);

    // Search Point
    let mut self_point = self_clan
        .point_select(&app_state.pool)
        .await
        .unwrap_or_default();
    self_point.clan_id = self_clan.id.unwrap_or_default();

    let mut rival_point = rival_clan
        .point_select(&app_state.pool)
        .await
        .unwrap_or_default();
    rival_point.clan_id = rival_clan.id.unwrap_or_default();

    // 先添加Track
    let track = Track::new(self_point, rival_point, &app_state.pool).await;
    let check_self_tracks = Track::select_desc_limit(track.self_clan_id, 1, &app_state.pool)
        .await
        .unwrap();
    let check_rival_tracks = Track::select_desc_limit(track.rival_clan_id, 1, &app_state.pool)
        .await
        .unwrap();

    // 预查限制重复登记
    if !check_self_tracks.is_empty() || !check_rival_tracks.is_empty() {
        log_warn!("预查重复登记");
        return (StatusCode::FORBIDDEN, Json(track));
    }

    // 数据库Unique限制重复
    let track_res = if let Ok(qr) = track.insert(&app_state.pool).await {
        qr.rows_affected()
    } else {
        log_warn!("数据库Unique重复");
        return (StatusCode::FORBIDDEN, Json(track));
    };

    // 更新self
    let self_point = ClanPoint::new(track.self_clan_id, track.self_now_point)
        .insert_or_update(&app_state.pool)
        .await
        .unwrap();

    // 更新rival
    let rival_point = ClanPoint::new(track.rival_clan_id, track.rival_now_point)
        .insert_or_update(&app_state.pool)
        .await
        .unwrap();

    log_info!(
        "self: {} | rival: {} | track: {track_res}",
        self_point.rows_affected(),
        rival_point.rows_affected()
    );
    (StatusCode::OK, Json(track))
}

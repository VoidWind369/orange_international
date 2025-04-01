mod clan;
mod clan_point;
mod round;
mod series;
mod track;

use crate::AppState;
use crate::util::un_authorization;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
pub use clan::Clan;
pub use round::Round;
use serde_json::Value;
pub use track::Track;
use uuid::Uuid;
use void_log::log_info;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/clan", get(clans).post(clan_insert))
        .route("/clan/{id}", get(clan))
        .route("/round", post(round_insert))
        .route("/new_track", post(new_track))
}

/// # All Clan
async fn clans(State(app_state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(vec![]));
    }
    // ********************鉴权********************

    let res = Clan::select_all(&app_state.pool).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn clan(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(Default::default()));
    }
    // ********************鉴权********************

    let res = Clan::select(&app_state.pool, id).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn clan_insert(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<Clan>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(-1));
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

async fn round_insert(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(-1));
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
    headers: HeaderMap,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(0));
    }
    // ********************鉴权********************

    let self_tag = data["self_tag"].as_str().unwrap_or_default();
    let rival_tag = data["rival_tag"].as_str().unwrap_or_default();
    let is_intel = if let Some(intel) = data.get("is_intel") {
        intel.as_bool().unwrap_or_default()
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

    // Search Point
    let self_point = self_clan
        .point_select(&app_state.pool)
        .await
        .unwrap_or_default();
    let rival_point = rival_clan
        .point_select(&app_state.pool)
        .await
        .unwrap_or_default();

    let track = Track::new(self_point, rival_point, &app_state.pool).await;
    let res = track.insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected as i64))
}

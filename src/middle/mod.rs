mod track;

use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use axum_auth::AuthBearer;
use chrono::Utc;
use void_log::{log_info, log_warn};
pub use track::*;
use crate::api::MiddleTrackApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/track/{tag}", get(track_tag))
}

async fn track_tag(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(tag): Path<String>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if !token.eq("middle*track*select") {
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************
    
    let res = Track::select_tag(&app_state.pool, &tag).await;
    if let Ok (r) = res {
        if Utc::now() - r.update_time > chrono::Duration::hours(1) {
            // 超过1h重新缓存
            let mta = MiddleTrackApi::get(&tag).await;
            let cache = mta.clone().self_to_database().update(&app_state.pool).await;
            if let Ok (r) = cache {
                log_info!("Update Cache {}", r.rows_affected());
                (StatusCode::OK, Json(mta))
            } else {
                log_warn!("MiddleTrackApi Update Cache Error");
                (StatusCode::GONE, Json::default())
            }
        } else {
            // 直接查缓存
            log_info!("{} has Cache", r.tag);
            (StatusCode::OK, Json(r.self_to_api()))
        }
    } else { 
        // 第一次查询新增
        let mta = MiddleTrackApi::get(&tag).await;
        let cache = mta.clone().self_to_database().insert(&app_state.pool).await;
        if let Ok (r) = cache { 
            log_info!("Create Cache {}", r.rows_affected());
            (StatusCode::OK, Json(mta))
        } else {
            log_warn!("MiddleTrackApi Create Cache Error");
            (StatusCode::GONE, Json::default())
        }
    }
}
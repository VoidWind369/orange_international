mod clan;
mod clan_point;
mod round;
mod series;
mod track;

use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

pub use clan::Clan;
pub use clan_point::ClanPoint;
pub use round::Round;
pub use series::Series;
pub use track::Track;

pub fn router(app: Router<AppState>) -> Router<AppState> {
    let app = app
        .route("/clan_insert", post(clan_insert))
        .route("/round_insert", post(round_insert));
    Router::new().nest("/orange", app)
}

async fn clan_api_insert(
    State(app_state): State<AppState>,
    Json(data): Json<Clan>,
) -> impl IntoResponse {
    let res = data.api_insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    Json(rows_affected)
}

async fn clan_insert(
    State(app_state): State<AppState>,
    Json(data): Json<Clan>,
) -> impl IntoResponse {
    let res = data.insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    Json(rows_affected)
}

async fn round_insert(
    State(app_state): State<AppState>,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    if let Some(time_str) = data["time"].as_str() {
        let res = Round::insert(time_str, &app_state.pool).await;
        let rows_affected = res.unwrap_or_default().rows_affected();
        (StatusCode::OK, Json(rows_affected))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(0))
    }
}

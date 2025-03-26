mod clan;
mod series;
mod clan_point;
mod round;
mod track;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::get;
use crate::AppState;

pub use clan::Clan;
pub use series::Series;

pub fn router(app: Router<AppState>) -> Router<AppState> {
    app.route("/system", get(|| async { "Is orange time!" }))
}

async fn clan_api_insert(State(app_state): State<AppState>, Json(data): Json<Clan>) -> impl IntoResponse {
    let res = data.api_insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    Json(rows_affected)
}

async fn clan_insert(State(app_state): State<AppState>, Json(data): Json<Clan>) -> impl IntoResponse {
    let res = data.insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    Json(rows_affected)
}
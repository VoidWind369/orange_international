use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::http::header::AUTHORIZATION;
use axum::routing::get;
use serde_json::{json, Value};
use crate::AppState;
use crate::system::user::User;
use crate::util::Config;

mod user;

pub fn router(app: Router) -> Router {
    app.route("/orange", get(|| async { "Is system time!" }))
}

async fn login(state: AppState, Json(data):Json<User>) -> impl IntoResponse {
    let pool = state.pool;
    let password = data.verify_login(&pool).await;
    Json(password)
}
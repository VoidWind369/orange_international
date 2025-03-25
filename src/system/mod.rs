use crate::system::user::User;
use crate::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

mod user;

pub fn router(app: Router) -> Router {
    app.route("/orange", get(|| async { "Is system time!" }))
}

async fn login(state: AppState, Json(data):Json<User>) -> impl IntoResponse {
    let pool = state.pool;
    let password = data.verify_login(&pool).await;
    Json(password)
}
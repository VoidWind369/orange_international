use crate::AppState;
use crate::system::user::User;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum::extract::State;
use crate::orange::Clan;

mod user;

pub fn router(app: Router<AppState>) -> Router<AppState> {
    app.route("/orange", get(|| async { "Is system time!" }))
        .route("/login", post(login))
}

async fn login(State(app_state): State<AppState>, Json(data): Json<User>) -> impl IntoResponse {
    let pool = app_state.pool;
    let password = data.verify_login(&pool).await;
    Json(password)
}

async fn user_insert(State(app_state): State<AppState>, Json(data): Json<User>) -> impl IntoResponse {
    let res = data.insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    Json(rows_affected)
}

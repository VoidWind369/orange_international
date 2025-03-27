use crate::system::user::User;
use crate::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use crate::util::un_authorization;

mod user;
mod redis;

pub fn router(app: Router<AppState>) -> Router<AppState> {
    app.route("/orange", get(|| async { "Is system time!" }))
        .route("/login", post(login))
        .route("/user_insert", post(user_insert))
}

async fn login(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<User>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(headers) {
        return (StatusCode::UNAUTHORIZED, Json(false));
    }
    // ********************鉴权********************

    let pool = app_state.pool;
    let password = data.verify_login(&pool).await;
    if password {

    }
    (StatusCode::OK, Json(password))
}

async fn user_insert(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<User>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(headers) {
        return (StatusCode::UNAUTHORIZED, Json(0));
    }
    // ********************鉴权********************

    let res = data.insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

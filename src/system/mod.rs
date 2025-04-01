use crate::AppState;
use crate::system::user::User;
use crate::util::un_authorization;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;
use void_log::log_info;

mod redis;
mod user;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/user", get(users).post(user_insert).put(user_update))
        .route("/user/{id}", get(user).delete(user_delete))
}

async fn login(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<User>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(false));
    }
    // ********************鉴权********************

    let pool = app_state.pool;
    let password = data.verify_login(&pool).await;
    if password {}
    (StatusCode::OK, Json(password))
}

async fn users(State(app_state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(vec![]));
    }
    // ********************鉴权********************

    let res = User::select_all(&app_state.pool).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn user(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(Default::default()));
    }
    // ********************鉴权********************

    let res = User::select(&app_state.pool, id).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn user_insert(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<User>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(0));
    }
    // ********************鉴权********************

    let res = data.insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

async fn user_update(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<User>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(0));
    }
    // ********************鉴权********************

    let res = data.update(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

async fn user_delete(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if un_authorization(&headers) {
        return (StatusCode::UNAUTHORIZED, Json(0));
    }
    // ********************鉴权********************

    let res = User::delete(&app_state.pool, id).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

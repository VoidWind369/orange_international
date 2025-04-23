use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, head};
use axum::{Json, Router};
use axum_auth::AuthBearer;
use uuid::Uuid;
use void_log::{log_info, log_warn};

mod group;
mod redis;
mod user;

pub use group::Group;
pub use redis::UserInfo;
pub use user::User;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", head(check_online).post(login).delete(logout))
        .route("/user", get(users).post(user_insert).put(user_update))
        .route("/user/{id}", get(user).delete(user_delete))
}

async fn login(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<User>,
) -> impl IntoResponse {
    log_info!("{}", &token);
    // ********************鉴权********************
    if !token.eq("cfa*login*auth") {
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let pool = app_state.pool;
    if let Some(check) = data.verify_login(&pool).await {
        (StatusCode::OK, Json(check))
    } else {
        (StatusCode::UNAUTHORIZED, Json::default())
    }
}

async fn logout(AuthBearer(token): AuthBearer) -> impl IntoResponse {
    log_info!("Logout: {}", &token);
    let res = if let Ok(_) = UserInfo::del_user(&token).await {
        1
    } else {
        0
    };
    Json(res)
}

async fn check_online(AuthBearer(token): AuthBearer) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        StatusCode::UNAUTHORIZED
    } else {
        StatusCode::OK
    }
    // ********************鉴权********************
}

async fn users(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = User::select_all(&app_state.pool).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn user(
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

    let res = User::select(&app_state.pool, id).await.unwrap();
    log_info!("{:?}", res);
    (StatusCode::OK, Json(res))
}

async fn user_insert(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<User>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = data.insert(&app_state.pool).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

async fn user_update(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(data): Json<User>,
) -> impl IntoResponse {
    // ********************鉴权********************
    if let Err(e) = UserInfo::get_user(&token).await {
        log_warn!("UNAUTHORIZED {e}");
        return (StatusCode::UNAUTHORIZED, Json::default());
    }
    // ********************鉴权********************

    let res = if data.password.is_some() {
        data.update_password(&app_state.pool).await
    } else if data.status.is_some() { 
        data.update_status(&app_state.pool).await
    } else {
        data.update(&app_state.pool).await
    };
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

async fn user_delete(
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

    let res = User::delete(&app_state.pool, id).await;
    let rows_affected = res.unwrap_or_default().rows_affected();
    (StatusCode::OK, Json(rows_affected))
}

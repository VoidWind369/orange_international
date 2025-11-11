use crate::AppState;
use crate::safety::authorization::VoidToken;
use crate::system::LoginLog;
use crate::util::RestApi;
use axum::Router;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum_msgpack::MsgPack;
use void_log::log_info;

mod authorization;

async fn get_token() -> impl IntoResponse {
    MsgPack(VoidToken::new_token())
}

async fn login_log(State(app_state): State<AppState>) -> impl IntoResponse {
    let data = LoginLog::select_all(&app_state.pool)
        .await
        .unwrap_or_default();
    RestApi::new("ok", "ok", Some(data)).builder_msgpack()
}

async fn get_login_log(
    State(app_state): State<AppState>,
    Path(text): Path<String>,
) -> impl IntoResponse {
    log_info!("Check {}", &text);
    let data = LoginLog::select_code_or_name(&app_state.pool, text)
        .await
        .unwrap_or_default();
    RestApi::new_successful(data).builder_msgpack()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get_token", get(get_token))
        .route("/login_log", get(login_log))
        .route("/login_log/{text}", get(get_login_log))
}

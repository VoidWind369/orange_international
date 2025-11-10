use crate::safety::authorization::VoidToken;
use crate::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum_msgpack::MsgPack;

mod authorization;

async fn get_token() -> impl IntoResponse {
    MsgPack(VoidToken::new_token())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get_token", get(get_token))
}
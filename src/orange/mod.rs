use axum::Router;
use axum::routing::get;
use crate::AppState;

pub fn router(app: Router<AppState>) -> Router<AppState> {
    app.route("/system", get(|| async { "Is orange time!" }))
}
pub mod handler;
pub mod json_rpc;
pub mod subscribe;

use axum::{routing::get, Router};
use handler::ws_handler;

use crate::server::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/wss", get(ws_handler))
}

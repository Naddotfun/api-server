pub mod handler;
pub mod path;
use axum::{routing::get, Router};

use handler::search_token;
use path::Path;

use crate::server::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route(Path::Search.as_str(), get(search_token))
}

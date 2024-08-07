pub mod handler;
pub mod path;
use axum::{routing::get, Router};
use path::Path;

use crate::server::state::AppState;
use handler::{
    get_coin_created, get_coin_hold, get_followers, get_followings, get_profile, get_replies,
};
pub fn router() -> Router<AppState> {
    Router::new()
        .route(Path::Main.as_str(), get(get_profile))
        .route(Path::CoinHeld.as_str(), get(get_coin_hold))
        .route(Path::Replies.as_str(), get(get_replies))
        // // .route(
        // //     Path::Notifications.as_str(),
        // //     get(handler::get_notifications),
        // // )
        .route(Path::CoinCreated.as_str(), get(get_coin_created))
        .route(Path::Followers.as_str(), get(get_followers))
        .route(Path::Followings.as_str(), get(get_followings))
}

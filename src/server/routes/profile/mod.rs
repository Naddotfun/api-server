pub mod handler;
pub mod path;
use crate::server::state::AppState;

use axum::{routing::get, Router};
use handler::{
    get_coins_held, get_created_coins, get_followers, get_following, get_profile, get_replies,
};
use path::ProfilePath;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(ProfilePath::Profile.as_str(), get(get_profile))
        .route(ProfilePath::CoinCreated.as_str(), get(get_coins_held))
        .route(ProfilePath::CoinHeld.as_str(), get(get_coins_held))
        .route(ProfilePath::Replies.as_str(), get(get_replies))
        .route(ProfilePath::Followers.as_str(), get(get_followers))
        .route(ProfilePath::Following.as_str(), get(get_following))
}

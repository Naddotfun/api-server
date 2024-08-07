pub mod handler;
pub mod path;
use crate::server::state::AppState;

use axum::{routing::get, Router};
use handler::{
    get_coins_held_by_address, get_coins_held_by_nickname, get_created_coins_by_address,
    get_created_coins_by_nickname, get_followers_by_address, get_followers_by_nickname,
    get_following_by_address, get_following_by_nickname, get_profile_by_address,
    get_profile_by_nickname, get_replies_by_address, get_replies_by_nickname,
};
use path::ProfilePath;

pub fn router() -> Router<AppState> {
    Router::new()
        // Profile routes
        .route(
            ProfilePath::ProfileByNickname.as_str(),
            get(get_profile_by_nickname),
        )
        .route(
            ProfilePath::ProfileByAddress.as_str(),
            get(get_profile_by_address),
        )
        // Coins held routes
        .route(
            ProfilePath::CoinHeldByNickname.as_str(),
            get(get_coins_held_by_nickname),
        )
        .route(
            ProfilePath::CoinHeldByAddress.as_str(),
            get(get_coins_held_by_address),
        )
        // Replies routes
        .route(
            ProfilePath::RepliesByNickname.as_str(),
            get(get_replies_by_nickname),
        )
        .route(
            ProfilePath::RepliesByAddress.as_str(),
            get(get_replies_by_address),
        )
        // Created coins routes
        .route(
            ProfilePath::CoinCreatedByNickname.as_str(),
            get(get_created_coins_by_nickname),
        )
        .route(
            ProfilePath::CoinCreatedByAddress.as_str(),
            get(get_created_coins_by_address),
        )
        // Followers routes
        .route(
            ProfilePath::FollowersByNickname.as_str(),
            get(get_followers_by_nickname),
        )
        .route(
            ProfilePath::FollowersByAddress.as_str(),
            get(get_followers_by_address),
        )
        // Following routes
        .route(
            ProfilePath::FollowingByNickname.as_str(),
            get(get_following_by_nickname),
        )
        .route(
            ProfilePath::FollowingByAddress.as_str(),
            get(get_following_by_address),
        )
}

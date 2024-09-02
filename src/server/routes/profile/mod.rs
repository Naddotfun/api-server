pub mod handler;
pub mod path;
use crate::server::state::AppState;

use axum::{routing::get, Router};
use handler::{
    get_created_tokens, get_followers, get_following, get_profile, get_replies, get_tokens_held,
};
use path::ProfilePath;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(ProfilePath::Profile.as_str(), get(get_profile))
        .route(ProfilePath::TokenCreated.as_str(), get(get_created_tokens))
        .route(ProfilePath::TokenHeld.as_str(), get(get_tokens_held))
        .route(ProfilePath::Replies.as_str(), get(get_replies))
        .route(ProfilePath::Followers.as_str(), get(get_followers))
        .route(ProfilePath::Following.as_str(), get(get_following))
}

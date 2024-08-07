use crate::{
    db::postgres::controller::profile::ProfileController,
    server::{result::AppJsonResult, state::AppState},
    types::{
        model::{Account, Coin, Thread},
        profile::{HoldCoin, Identifier},
    },
};

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

use utoipa::ToSchema;

use super::path::ProfilePath;

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProfileResponse {
    account: Account,
}
#[derive(ToSchema, Serialize)]
pub struct HeldCoinsResponse {
    coins: Vec<HoldCoin>,
}
#[derive(ToSchema, Serialize)]
pub struct RepliesResponse {
    replies: Vec<Thread>,
}
#[derive(ToSchema, Serialize)]
pub struct CreatedCoinsResponse {
    coins: Vec<Coin>,
}

#[derive(ToSchema, Serialize)]
pub struct FollowersResponse {
    followers: Vec<Account>,
}

#[derive(ToSchema, Serialize)]
pub struct FollowingResponse {
    following: Vec<Account>,
}

fn is_address(user: &str) -> bool {
    // 주소 형식 검증 로직 (예: 0x로 시작하고 적절한 길이인지 확인)
    user.starts_with("0x") && user.len() == 42
}

/// Get user profile
#[utoipa::path(
    get,
    path = ProfilePath::Profile.docs_str(),
    params(
        ("user" = String, Path, description = "User's nickname or Ethereum address")
    ),
    responses(
        (status = 200, description = "User profile retrieved successfully", body = ProfileResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile"
)]
pub async fn get_profile(
    Path(user): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<ProfileResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let identifier = if is_address(&user) {
        Identifier::Address(user)
    } else {
        Identifier::Nickname(user)
    };
    let account = profile_controller.get_profile(&identifier).await?;
    Ok(Json(ProfileResponse { account }))
}

/// Get user's held coins
#[utoipa::path(
    get,
    path = ProfilePath::CoinHeld.docs_str(),
    params(
        ("user" = String, Path, description = "User's nickname or Ethereum address")
    ),
    responses(
        (status = 200, description = "User's held coins retrieved successfully", body = HeldCoinsResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile"
)]
pub async fn get_coins_held(
    Path(user): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<HeldCoinsResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let identifier = if is_address(&user) {
        Identifier::Address(user)
    } else {
        Identifier::Nickname(user)
    };
    let coins = profile_controller.get_holding_coin(&identifier).await?;
    Ok(Json(HeldCoinsResponse { coins }))
}

/// Get user's replies
#[utoipa::path(
    get,
    path = ProfilePath::Replies.docs_str(),
    params(
        ("user" = String, Path, description = "User's nickname or Ethereum address")
    ),
    responses(
        (status = 200, description = "User's replies retrieved successfully", body = RepliesResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile"
)]
pub async fn get_replies(
    Path(user): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<RepliesResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let identifier = if is_address(&user) {
        Identifier::Address(user)
    } else {
        Identifier::Nickname(user)
    };
    let replies = profile_controller.get_replies(&identifier).await?;
    Ok(Json(RepliesResponse { replies }))
}

/// Get coins created by user
#[utoipa::path(
    get,
    path = ProfilePath::CoinCreated.docs_str(),
    params(
        ("user" = String, Path, description = "User's nickname or Ethereum address")
    ),
    responses(
        (status = 200, description = "Created coins retrieved successfully", body = CreatedCoinsResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile"
)]
pub async fn get_created_coins(
    Path(user): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<CreatedCoinsResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let identifier = if is_address(&user) {
        Identifier::Address(user)
    } else {
        Identifier::Nickname(user)
    };
    let coins = profile_controller.get_created_coins(&identifier).await?;
    Ok(Json(CreatedCoinsResponse { coins }))
}

/// Get user's followers
#[utoipa::path(
    get,
    path = ProfilePath::Followers.docs_str(),
    params(
        ("user" = String, Path, description = "User's nickname or Ethereum address")
    ),
    responses(
        (status = 200, description = "User's followers retrieved successfully", body = FollowersResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile"
)]
pub async fn get_followers(
    Path(user): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<FollowersResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let identifier = if is_address(&user) {
        Identifier::Address(user)
    } else {
        Identifier::Nickname(user)
    };
    let followers = profile_controller.get_followers(&identifier).await?;
    Ok(Json(FollowersResponse { followers }))
}

/// Get accounts followed by user
#[utoipa::path(
    get,
    path = ProfilePath::Following.docs_str(),
    params(
        ("user" = String, Path, description = "User's nickname or Ethereum address")
    ),
    responses(
        (status = 200, description = "Followed accounts retrieved successfully", body = FollowingResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile"
)]
pub async fn get_following(
    Path(user): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<FollowingResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let identifier = if is_address(&user) {
        Identifier::Address(user)
    } else {
        Identifier::Nickname(user)
    };
    let following = profile_controller.get_following(&identifier).await?;
    Ok(Json(FollowingResponse { following }))
}

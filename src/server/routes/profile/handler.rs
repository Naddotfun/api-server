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

/// Get user profile by nickname
#[utoipa::path(
    get,
    path = ProfilePath::ProfileByNickname.docs_str(),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    ),
    responses(
        (status = 200, description = "User profile retrieved successfully", body = ProfileResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Nickname"
)]
pub async fn get_profile_by_nickname(
    Path(nickname): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<ProfileResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let account = profile_controller
        .get_profile(&Identifier::Nickname(nickname))
        .await?;
    Ok(Json(ProfileResponse { account }))
}

/// Get user profile by Ethereum address
#[utoipa::path(
    get,
    path = ProfilePath::ProfileByAddress.docs_str(),
    params(
        ("address" = String, Path, description = "User's Ethereum address ")
    ),
    responses(
        (status = 200, description = "User profile retrieved successfully", body = ProfileResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Address"
)]
pub async fn get_profile_by_address(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<ProfileResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let account = profile_controller
        .get_profile(&Identifier::Address(address))
        .await?;
    Ok(Json(ProfileResponse { account }))
}

/// Get user's held coins by nickname
#[utoipa::path(
    get,
    path = ProfilePath::CoinHeldByNickname.docs_str(),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    ),
    responses(
        (status = 200, description = "User's held coins retrieved successfully", body = HeldCoinsResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
     tag = "Profile Nickname"
)]
pub async fn get_coins_held_by_nickname(
    Path(nickname): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<HeldCoinsResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let coins = profile_controller
        .get_holding_coin(&Identifier::Nickname(nickname))
        .await?;
    Ok(Json(HeldCoinsResponse { coins }))
}

/// Get user's held coins by Ethereum address
#[utoipa::path(
    get,
    path = ProfilePath::CoinHeldByAddress.docs_str(),
    params(
        ("address" = String, Path, description = "User's Ethereum address (0x prefixed)")
    ),
    responses(
        (status = 200, description = "User's held coins retrieved successfully", body = HeldCoinsResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Address"
)]
pub async fn get_coins_held_by_address(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<HeldCoinsResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let coins = profile_controller
        .get_holding_coin(&Identifier::Address(address))
        .await?;
    Ok(Json(HeldCoinsResponse { coins }))
}

/// Get user's replies by nickname
#[utoipa::path(
    get,
    path = ProfilePath::RepliesByNickname.docs_str(),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    ),
    responses(
        (status = 200, description = "User's replies retrieved successfully", body = RepliesResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Nickname"
)]
pub async fn get_replies_by_nickname(
    Path(nickname): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<RepliesResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let replies = profile_controller
        .get_replies(&Identifier::Nickname(nickname))
        .await?;
    Ok(Json(RepliesResponse { replies }))
}

/// Get user's replies by Ethereum address
#[utoipa::path(
    get,
    path = ProfilePath::RepliesByAddress.docs_str(),
    params(
        ("address" = String, Path, description = "User's Ethereum address (0x prefixed)")
    ),
    responses(
        (status = 200, description = "User's replies retrieved successfully", body = RepliesResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Address"
)]
pub async fn get_replies_by_address(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<RepliesResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let replies = profile_controller
        .get_replies(&Identifier::Address(address))
        .await?;
    Ok(Json(RepliesResponse { replies }))
}

/// Get coins created by user (by nickname)
#[utoipa::path(
    get,
    path = ProfilePath::CoinCreatedByNickname.docs_str(),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    ),
    responses(
        (status = 200, description = "Created coins retrieved successfully", body = CreatedCoinsResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Nickname"
)]
pub async fn get_created_coins_by_nickname(
    Path(nickname): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<CreatedCoinsResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let coins = profile_controller
        .get_created_coins(&Identifier::Nickname(nickname))
        .await?;
    Ok(Json(CreatedCoinsResponse { coins }))
}

/// Get coins created by user (by Ethereum address)
#[utoipa::path(
    get,
    path = ProfilePath::CoinCreatedByAddress.docs_str(),
    params(
        ("address" = String, Path, description = "User's Ethereum address (0x prefixed)")
    ),
    responses(
        (status = 200, description = "Created coins retrieved successfully", body = CreatedCoinsResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Address"
)]
pub async fn get_created_coins_by_address(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<CreatedCoinsResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let coins = profile_controller
        .get_created_coins(&Identifier::Address(address))
        .await?;
    Ok(Json(CreatedCoinsResponse { coins }))
}

/// Get user's followers by nickname
#[utoipa::path(
    get,
    path = ProfilePath::FollowersByNickname.docs_str(),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    ),
    responses(
        (status = 200, description = "User's followers retrieved successfully", body = FollowersResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Nickname"
)]
pub async fn get_followers_by_nickname(
    Path(nickname): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<FollowersResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let followers = profile_controller
        .get_followers(&Identifier::Nickname(nickname))
        .await?;
    Ok(Json(FollowersResponse { followers }))
}

/// Get user's followers by Ethereum address
#[utoipa::path(
    get,
    path = ProfilePath::FollowersByAddress.docs_str(),
    params(
        ("address" = String, Path, description = "User's Ethereum address (0x prefixed)")
    ),
    responses(
        (status = 200, description = "User's followers retrieved successfully", body = FollowersResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Address"
)]
pub async fn get_followers_by_address(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<FollowersResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let followers = profile_controller
        .get_followers(&Identifier::Address(address))
        .await?;
    Ok(Json(FollowersResponse { followers }))
}

/// Get accounts followed by user (by nickname)
#[utoipa::path(
    get,
    path = ProfilePath::FollowingByNickname.docs_str(),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    ),
    responses(
        (status = 200, description = "Followed accounts retrieved successfully", body = FollowingResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Nickname"
)]
pub async fn get_following_by_nickname(
    Path(nickname): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<FollowingResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let following = profile_controller
        .get_following(&Identifier::Nickname(nickname))
        .await?;
    Ok(Json(FollowingResponse { following }))
}

/// Get accounts followed by user (by Ethereum address)
#[utoipa::path(
    get,
    path = ProfilePath::FollowingByAddress.docs_str(),
    params(
        ("address" = String, Path, description = "User's Ethereum address (0x prefixed)")
    ),
    responses(
        (status = 200, description = "Followed accounts retrieved successfully", body = FollowingResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile Address"
)]
pub async fn get_following_by_address(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<FollowingResponse> {
    let profile_controller = ProfileController::new(state.postgres.clone());
    let following = profile_controller
        .get_following(&Identifier::Address(address))
        .await?;
    Ok(Json(FollowingResponse { following }))
}

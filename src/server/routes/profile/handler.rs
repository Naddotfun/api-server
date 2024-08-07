use super::path::Path as ProfilePath;
use crate::{
    db::postgres::controller::profile::ProfileController,
    server::{result::AppJsonResult, state::AppState},
    types::{
        model::{Account, Coin, Thread},
        HoldCoinResponse,
    },
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(serde::Serialize, ToSchema)]
pub struct ProfileResponse {
    account: Account,
}

#[utoipa::path(
    get,
    path = ProfilePath::Main.as_str(),
    responses(
        (status = 200, description = "Get user profile", body = ProfileResponse),
        (status = 404, description = "User not found")
    ),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    )
)]
pub async fn get_profile(
    Path(nickname): Path<String>,
    state: axum::extract::State<AppState>,
) -> AppJsonResult<ProfileResponse> {
    let profile_contoller = ProfileController::new(state.postgres.clone());
    let account = profile_contoller.get_profile(&nickname).await?;

    Ok(Json(ProfileResponse { account }))
}

#[utoipa::path(
    get,
    path = ProfilePath::CoinHeld.as_str(),
    responses(
        (status = 200, description = "Get address holding tokens", body = ProfileResponse),
        (status = 404, description = "User not found")
    ),
    params(
        ("nickname" = String, Path, description = "User's nickname")
    )
)]

pub async fn get_coin_hold(
    Path(nickname): Path<String>,
    state: axum::extract::State<AppState>,
) -> AppJsonResult<Vec<HoldCoinResponse>> {
    let profile_contoller = ProfileController::new(state.postgres.clone());
    let hold_response = profile_contoller
        .get_account_holding_coin(&nickname)
        .await?;

    Ok(Json(hold_response))
}
#[derive(serde::Serialize, ToSchema)]
pub struct RepliesResponse {
    replies: Vec<Thread>,
}
pub async fn get_replies(
    Path(address): Path<String>,
    state: State<AppState>,
) -> AppJsonResult<RepliesResponse> {
    let profile_contoller = ProfileController::new(state.postgres.clone());
    let replies = profile_contoller.get_account_replies(&address).await?;

    Ok(Json(RepliesResponse { replies }))
}
#[derive(serde::Serialize, ToSchema)]
pub struct CreateCoinResponse {
    coin: Vec<Coin>,
}

pub async fn get_coin_created(
    Path(address): Path<String>,
    state: State<AppState>,
) -> AppJsonResult<CreateCoinResponse> {
    let profile_contoller = ProfileController::new(state.postgres.clone());
    let coins = profile_contoller
        .get_account_created_coins(&address)
        .await?;

    Ok(Json(CreateCoinResponse { coin: coins }))
}
#[derive(serde::Serialize, ToSchema)]
pub struct FollowersReponse {
    followers: Vec<Account>,
}

pub async fn get_followers(
    Path(address): Path<String>,
    state: State<AppState>,
) -> AppJsonResult<FollowersReponse> {
    let profile_contoller = ProfileController::new(state.postgres.clone());
    let followers = profile_contoller.get_account_followers(&address).await?;

    Ok(Json(FollowersReponse { followers }))
}
#[derive(serde::Serialize, ToSchema)]
pub struct FollowingResponse {
    following: Vec<Account>,
}

pub async fn get_followings(
    Path(address): Path<String>,
    state: State<AppState>,
) -> AppJsonResult<FollowingResponse> {
    let profile_contoller = ProfileController::new(state.postgres.clone());
    let followings = profile_contoller.get_account_follwing(&address).await?;

    Ok(Json(FollowingResponse {
        following: followings,
    }))
}

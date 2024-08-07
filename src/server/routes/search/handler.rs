use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::ToSchema;

use super::path::Path;
use crate::db::postgres::controller::order::OrderController;
use crate::server::result::AppJsonResult;
use crate::types::event::order::OrderTokenResponse;
use crate::{server::state::AppState, types::model::Coin};
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "address": "Your address"
}))]
pub struct SearchRequest {
    #[schema(example = "Your address")]
    token_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "token_name": "coin name or symbol"
}))]
pub struct SearchResponse {
    // #[schema(example = "token_name or symbol")]
    tokens: Vec<OrderTokenResponse>,
}
#[utoipa::path(
    post,
    path = Path::Search.as_str(),
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Nonce generated successfully", body = SearchResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
#[instrument(skip(state))]
pub async fn search_token(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> AppJsonResult<SearchResponse> {
    let query = payload.token_name;
    // `session_address`를 사용하여 인증된 사용자의 주소에 접근
    let order_contoller = OrderController::new(state.postgres.clone());
    let tokens = order_contoller.search_order_tokens(&query).await?;
    Ok(Json(SearchResponse { tokens }))
}

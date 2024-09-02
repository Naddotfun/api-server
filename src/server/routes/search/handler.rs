use axum::extract::Path;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use utoipa::ToSchema;

use super::path::Path as SearchPath;
use crate::db::postgres::controller::order::OrderController;
use crate::server::result::AppJsonResult;
use crate::types::event::order::OrderTokenResponse;
use crate::{server::state::AppState, types::model::Token};

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "token_name": "token name or symbol"
}))]
pub struct SearchResponse {
    // #[schema(example = "token_name or symbol")]
    tokens: Vec<OrderTokenResponse>,
}
#[utoipa::path(
    get,
    path = SearchPath::Search.docs_str(),
    params(
        ("token" = String, Path, description = "Token to search for")
    ),
    responses(
        (status = 200, description = "Nonce generated successfully", body = SearchResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Search Token"
)]
#[instrument(skip(state))]
pub async fn search_token(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> AppJsonResult<SearchResponse> {
    let order_contoller = OrderController::new(state.postgres.clone());
    let tokens = order_contoller.search_order_tokens(&token).await?;
    Ok(Json(SearchResponse { tokens }))
}

pub mod result;
pub mod routes;
pub mod state;

use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use axum::{
    error_handling::HandleErrorLayer,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    BoxError, Router,
};


use routes::{
    profile::{
        self,
        handler::{
            CreatedTokensResponse, FollowersResponse, FollowingResponse, HeldTokensResponse,
            ProfileResponse, RepliesResponse,
        },
    },
    search::{self, handler::SearchResponse},
    socket
};

use state::AppState;
use tower::ServiceBuilder;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    db::{postgres::PostgresDatabase, redis::RedisDatabase},
    event::{new_content::NewContentEventProducer, order::OrderEventProducer, token::TokenEventProducer},
    types::{
        event::{order::OrderTokenResponse, UserInfo},
        model::{Account, Thread, Token},
        profile::HoldToken,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        search::handler::search_token,
        profile::handler::get_profile,
        profile::handler::get_tokens_held,
        profile::handler::get_replies,
        profile::handler::get_created_tokens,
        profile::handler::get_followers,
        profile::handler::get_following,
    ),
    components(
        schemas(
            ProfileResponse,
            HeldTokensResponse,
            RepliesResponse,
            CreatedTokensResponse,
            FollowersResponse,
            FollowingResponse,
            Account,
            Token,
            HoldToken,
            Thread,
            SearchResponse,
            OrderTokenResponse,
            UserInfo
            
        )
    ),
    tags(
        (name = "Search Token", description = "Search token by name"),
        (name = "Profile", description = "Get information about a user by Nickname"),
        
    )
)]
pub struct ApiDoc;

pub async fn main(
    postgres: Arc<PostgresDatabase>,
    redis: Arc<RedisDatabase>,
    order_event_producer: Arc<OrderEventProducer>,
    token_event_producer: Arc<TokenEventProducer>,
    new_content_producer:Arc<NewContentEventProducer>
) -> Result<()> {
    let ip = std::env::var("IP").unwrap();
    let port = std::env::var("PORT").unwrap();
    let state = AppState {
        postgres,
        redis,
        order_event_producer,
        token_event_producer,
        new_content_producer
    };
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(socket::router())
        .merge(search::router())
        .merge(profile::router())
        // .merge(test::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(5)),
        )
        .with_state(state)
        .fallback(handler_404);

    let addr = SocketAddr::from((
        IpAddr::from_str(ip.as_str()).unwrap(),
        port.parse().unwrap(),
    ));
    info!("Listening on {} Server port{}", addr, port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn handle_timeout_error(
    // `Method` and `Uri` are extractors so they can be used here
    method: Method,
    uri: Uri,
    // the last argument must be the error itself
    err: BoxError,
) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("`{method} {uri}` failed with {err}"),
    )
}

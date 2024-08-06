pub mod routes;
pub mod socket;
pub mod state;
use std::{
    borrow::Cow,
    net::{IpAddr, SocketAddr},
    ops::ControlFlow,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    BoxError, Router,
};
use axum_extra::{headers, TypedHeader};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use socket::handle_socket;
use state::AppState;
use tower::ServiceBuilder;
use tracing::info;

use crate::{
    db::{postgres::PostgresDatabase, redis::RedisDatabase},
    event::{coin::CoinEventProducer, order::OrderEventProducer},
};

pub async fn main(
    postgres: Arc<PostgresDatabase>,
    redis: Arc<RedisDatabase>,
    order_event_producer: Arc<OrderEventProducer>,
    coin_event_producer: Arc<CoinEventProducer>,
) -> Result<()> {
    let ip = std::env::var("IP").unwrap();
    let port = std::env::var("PORT").unwrap();
    let state = AppState {
        postgres,
        redis,
        order_event_producer,
        coin_event_producer,
    };
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", get(ws_handler))
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        info!("User-Agent: {}", user_agent);
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
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

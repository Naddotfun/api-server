pub mod handler;
use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};

use axum_extra::{headers, TypedHeader};
use futures::{SinkExt, StreamExt};
use handler::ws_handler;
use json_rpc::{send_error_response, JsonRpcErrorCode, JsonRpcMethod, JsonRpcRequest};
use subscribe::{handle_coin_subscribe, handle_order_subscribe};
use tokio::{
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    task::JoinHandle,
};
use tracing::{error, info};

use crate::server::state::AppState;

pub mod json_rpc;
pub mod subscribe;

pub fn router() -> Router<AppState> {
    Router::new().route("/wss", get(ws_handler))
}

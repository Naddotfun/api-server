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

use tokio::{
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    task::JoinHandle,
};
use tracing::{error, info};

use crate::server::{
    routes::socket::{
        json_rpc::{send_error_response, JsonRpcErrorCode},
        subscribe::handle_new_content_subscribe,
    },
    state::AppState,
};

use super::{
    json_rpc::{JsonRpcMethod, JsonRpcRequest},
    subscribe::{handle_order_subscribe, handle_token_subscribe},
};
enum ActiveSubscription {
    Order(JoinHandle<()>),
    Token(JoinHandle<()>),
    NewContent(JoinHandle<()>),
    None,
}
pub async fn ws_handler(
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

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

pub async fn handle_socket(socket: WebSocket, addr: SocketAddr, state: AppState) {
    info!("New WebSocket connection: {}", addr);
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(100);
    let (close_tx, close_rx) = oneshot::channel();

    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        let mut active_subscription = ActiveSubscription::None;
        while let Some(Ok(message)) = receiver.next().await {
            if let Err(e) =
                handle_message(message, &state_clone, &tx_clone, &mut active_subscription).await
            {
                if let Err(send_err) =
                    send_error_response(&tx_clone, JsonRpcErrorCode::InternalError, &e.to_string())
                        .await
                {
                    error!("Failed to send error response: {:?}", send_err);
                    break;
                }
            }
        }
        // Cancel the active subscription if any
        if let ActiveSubscription::Order(handle)
        | ActiveSubscription::Token(handle)
        | ActiveSubscription::NewContent(handle) = active_subscription
        {
            handle.abort();
        }
        let _ = close_tx.send(());
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
        _ = close_rx => {
            send_task.abort();
            recv_task.abort();
        }
    };

    info!("WebSocket connection closed: {}", addr);
}

async fn handle_message(
    msg: Message,
    state: &AppState,
    tx: &Sender<Message>,
    active_subscription: &mut ActiveSubscription,
) -> Result<()> {
    match msg {
        Message::Text(text) => {
            info!("Received message: {}", text);
            let request: JsonRpcRequest =
                serde_json::from_str(&text).context("Failed to parse JSON-RPC request")?;
            //request 를 method 로 바꿔보자.
            info!("Request: {:?}", request);
            match request.method() {
                JsonRpcMethod::OrderSubscribe => {
                    // Cancel any existing subscription
                    if let ActiveSubscription::Order(handle) =
                        std::mem::replace(active_subscription, ActiveSubscription::None)
                    {
                        handle.abort();
                    }
                    let new_handle = handle_order_subscribe(request, state, tx.clone()).await?;
                    *active_subscription = ActiveSubscription::Order(new_handle);
                    Ok(())
                }
                JsonRpcMethod::TokenSubscribe => {
                    // Cancel any existing subscription
                    if let ActiveSubscription::Token(handle) =
                        std::mem::replace(active_subscription, ActiveSubscription::None)
                    {
                        handle.abort();
                    }
                    let new_handle = handle_token_subscribe(request, state, tx.clone()).await?;
                    *active_subscription = ActiveSubscription::Token(new_handle);
                    Ok(())
                }
                JsonRpcMethod::NewContentSubscribe => {
                    if let ActiveSubscription::NewContent(handle) =
                        std::mem::replace(active_subscription, ActiveSubscription::None)
                    {
                        handle.abort();
                    }
                    let new_handle =
                        handle_new_content_subscribe(request, state, tx.clone()).await?;
                    *active_subscription = ActiveSubscription::NewContent(new_handle);
                    Ok(())
                }
                _ => {
                    send_error_response(tx, JsonRpcErrorCode::MethodNotFound, "Unknown method")
                        .await
                }
            }
        }
        _ => Err(anyhow::anyhow!("Unsupported message type")),
    }
}

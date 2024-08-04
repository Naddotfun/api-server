use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use axum::extract::ws::Message;
use serde_json::json;
use serde_json::Value;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tracing::error;

use crate::server::state::AppState;
use crate::types::event::order::OrderType;

use super::json_rpc::send_success_response;
use super::json_rpc::JsonRpcRequest;

pub async fn handle_order_subscribe(
    request: JsonRpcRequest,
    state: &AppState,
    tx: Sender<Message>,
) -> Result<JoinHandle<()>> {
    let order_type = parse_order_type(&request.params)
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing order type"))?;

    let order = state
        .redis
        .get_order(order_type)
        .await
        .context("Failed to get initial order")?;
    let order_json = serde_json::to_value(order).context("Failed to serialize order")?;

    let order_response = json!({
        "order": order_json,
    });
    send_success_response(&tx, &request.method, json!(order_response)).await?;

    let mut receiver = state
        .order_event_producer
        .get_order_receiver(order_type)
        .await;

    let handle = tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            if let Err(e) = send_success_response(&tx, &request.method, json!(event)).await {
                error!("Failed to send order event: {:?}", e);
                break;
            }
        }
        // Ensure receiver is dropped here
        drop(receiver);
    });

    Ok(handle)
}

pub async fn handle_coin_subscribe(
    request: JsonRpcRequest,
    state: &AppState,
    tx: Sender<Message>,
) -> Result<JoinHandle<()>> {
    let coin_id = parse_coin_id(&request.params)
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing coin ID"))?;

    let mut receiver = state.coin_event_producer.get_coin_receiver(&coin_id).await;

    send_success_response(
        &tx,
        &request.method,
        json!({"status": "subscribed", "coin_id": coin_id}),
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            if let Err(e) = send_success_response(&tx, &request.method, json!(event)).await {
                error!("Failed to send coin event: {:?}", e);
                break;
            }
        }
        // Ensure receiver is dropped here
        drop(receiver);
    });

    Ok(handle)
}

fn parse_order_type(params: &Option<Value>) -> Option<OrderType> {
    match params {
        Some(Value::String(s)) => OrderType::from_str(s).ok(),
        Some(Value::Object(obj)) => obj
            .get("order_type")
            .and_then(|v| v.as_str())
            .and_then(|s| OrderType::from_str(s).ok()),
        _ => None,
    }
}

fn parse_coin_id(params: &Option<Value>) -> Option<String> {
    match params {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(obj)) => obj
            .get("coin_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned()),
        _ => None,
    }
}

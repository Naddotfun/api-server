use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use axum::extract::ws::Message;
use serde_json::json;
use serde_json::Value;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tracing::error;
use tracing::info;

use crate::db::postgres::controller::coinpage::CoinPageController;
use crate::db::postgres::controller::message;
use crate::server::state::AppState;
use crate::types::event::coin_message::CoinMessage;
use crate::types::event::order::OrderMessage;
use crate::types::event::order::OrderType;
use crate::types::event::SendMessageType;

use super::json_rpc::send_success_response;
use super::json_rpc::JsonRpcRequest;

pub async fn handle_order_subscribe(
    request: JsonRpcRequest,
    state: &AppState,
    tx: Sender<Message>,
) -> Result<JoinHandle<()>> {
    let order_type = parse_order_type(request.params())
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing order type"))?;

    let order = state
        .redis
        .get_order(order_type)
        .await
        .context("Failed to get initial order")?;

    let new_token = state
        .redis
        .get_new_token()
        .await
        .context("Failed to get new token")?;
    info!("new_token: {:?}", new_token);
    // let new_swap = state
    //     .redis
    //     .get_new_swap()
    //     .await
    //     .context("Failed to get new_swap")?;
    let new_buy = state
        .redis
        .get_new_buy()
        .await
        .context("Failed to get new_swap")?;
    let new_sell = state
        .redis
        .get_new_sell()
        .await
        .context("Failed to get new_swap")?;
    info!("new_token: {:?}", new_token);
    let message = OrderMessage {
        message_type: SendMessageType::ALL,
        new_token,
        new_buy,
        new_sell,
        order_type,
        order_token: Some(order),
    };

    let order_json = serde_json::to_value(message).context("Failed to serialize order")?;

    send_success_response(&tx, request.method(), json!(order_json)).await?;

    let mut receiver = state
        .order_event_producer
        .get_order_receiver(order_type)
        .await;

    let handle = tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            if let Err(e) = send_success_response(&tx, request.method(), json!(event)).await {
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
    let coin_id = parse_coin_id(request.params())
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing coin ID"))?;

    let mut receiver = state.coin_event_producer.get_coin_receiver(&coin_id).await;

    //이제 여기서 부터 coin 에 대한 데이터를 가지고 와서 보내주는 코드 작성해야함.

    //먼저 레디스에 접근해서 coin_id 에 해당하는 coin 정보 가져옴.
    //없으면 postgres 에서 가져와서 redis 에 저장함.

    let new_token = state
        .redis
        .get_new_token()
        .await
        .context("Failed to get new token")?;
    let new_buy = state
        .redis
        .get_new_buy()
        .await
        .context("Failed to get new_swap")?;
    let new_sell = state
        .redis
        .get_new_sell()
        .await
        .context("Failed to get new_swap")?;
    let coin_page_controller = CoinPageController::new(state.postgres.clone());

    let coin_data = coin_page_controller.get_coin_message(&coin_id).await?;
    info!("coin_data _ price {:?}", coin_data.curve);
    let message = CoinMessage {
        message_type: SendMessageType::ALL,
        new_token,
        new_buy,
        new_sell,
        coin: coin_data,
    };
    let message_json = serde_json::to_value(message).context("Failed to serialize coin")?;
    send_success_response(
        &tx,
        request.method(),
        json!({"status": "subscribed","data": message_json}),
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            if let Err(e) = send_success_response(&tx, request.method(), json!(event)).await {
                error!("Failed to send coin event: {:?}", e);
                break;
            }
        }
        // Ensure receiver is dropped here
        drop(receiver);
    });

    Ok(handle)
}

fn parse_order_type(params: Option<&Value>) -> Option<OrderType> {
    match params {
        Some(Value::String(s)) => OrderType::from_str(s).ok(),
        Some(Value::Object(obj)) => obj
            .get("order_type")
            .and_then(|v| v.as_str())
            .and_then(|s| OrderType::from_str(s).ok()),
        _ => None,
    }
}

fn parse_coin_id(params: Option<&Value>) -> Option<String> {
    match params {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(obj)) => obj
            .get("coin_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned()),
        _ => None,
    }
}

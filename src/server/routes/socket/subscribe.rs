use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use axum::extract::ws::Message;
use serde_json::json;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;

use crate::db::postgres::controller::coinpage::CoinPageController;

use crate::server::state::AppState;

use crate::types::chart_type::ChartType;
use crate::types::event::coin::CoinMessage;
use crate::types::event::new_content::NewContentMessage;
use crate::types::event::order::OrderMessage;
use crate::types::event::order::OrderType;
use crate::types::event::NewSwapMessage;
use crate::types::event::NewTokenMessage;
use crate::types::event::SendMessageType;

use super::json_rpc::send_success_response;
use super::json_rpc::JsonRpcRequest;
struct NewContent {
    pub new_token: Option<NewTokenMessage>,
    pub new_buy: Option<NewSwapMessage>,
    pub new_sell: Option<NewSwapMessage>,
}

impl NewContent {
    pub async fn new(state: &AppState) -> Self {
        let new_token = match state.redis.get_new_token().await {
            Ok(token) => token,
            Err(e) => {
                error!("Failed to get new token: {:?}", e);
                None
            }
        };

        let new_buy = match state.redis.get_new_buy().await {
            Ok(buy) => buy,
            Err(e) => {
                error!("Failed to get new buy: {:?}", e);
                None
            }
        };

        let new_sell = match state.redis.get_new_sell().await {
            Ok(sell) => sell,
            Err(e) => {
                error!("Failed to get new sell: {:?}", e);
                None
            }
        };

        Self {
            new_token,
            new_buy,
            new_sell,
        }
    }
}

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

    let NewContent {
        new_token,
        new_buy,
        new_sell,
    } = NewContent::new(state).await;

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
        let subscribed_order_type = order_type.clone();
        while let Some(mut event) = receiver.recv().await {
            if event.order_type != subscribed_order_type {
                event.order_token = None;
            }
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
    info!("Coin subscribe");
    let coin_id = parse_coin_id(request.params())
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing coin ID"))?;
    let chart_type = ChartType::from_str(
        &parse_chart(request.params())
            .ok_or_else(|| anyhow::anyhow!("Invalid or missing chart"))?,
    )
    .map_err(|e| anyhow::anyhow!("Failed to parse chart type: {}", e))?;
    // info!("Chart type ={:?}", chart_type);

    //이제 여기서 부터 coin 에 대한 데이터를 가지고 와서 보내주는 코드 작성해야함.

    //먼저 레디스에 접근해서 coin_id 에 해당하는 coin 정보 가져옴.
    //없으면 postgres 에서 가져와서 redis 에 저장함.

    let NewContent {
        new_token,
        new_buy,
        new_sell,
    } = NewContent::new(state).await;

    let coin_page_controller = CoinPageController::new(state.postgres.clone());

    let coin_data = coin_page_controller
        .get_coin_message(&coin_id, chart_type.clone())
        .await?;
    // info!("Coin data is :{:?}", coin_data);
    let message = CoinMessage {
        message_type: SendMessageType::ALL,
        new_token,
        new_buy,
        new_sell,
        coin: coin_data,
    };
    let message_json = serde_json::to_value(message).context("Failed to serialize coin")?;
    send_success_response(&tx, request.method(), json!(message_json)).await?;

    // 메시지 수신 부분

    let mut receiver = state.coin_event_producer.get_coin_receiver(&coin_id).await;
    let handle = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            debug!("Received new coin message");

            let should_send = match &message.coin.chart {
                None => {
                    debug!("Chart is None, sending message");
                    true
                }
                Some(charts) => {
                    let matching_chart = charts
                        .iter()
                        .any(|cw| cw.chart_type == chart_type.to_string());
                    if matching_chart {
                        debug!("Found matching chart type, sending message");
                        true
                    } else {
                        debug!("No matching chart type, skipping message");
                        false
                    }
                }
            };

            if should_send {
                if let Err(e) = send_success_response(&tx, request.method(), json!(message)).await {
                    error!("Failed to send coin event: {:?}", e);
                    break;
                }
            }
        }
    });
    // info!("Receiver loop ended for coin_id: {}", coin_id);
    Ok(handle)
}

pub async fn handle_new_content_subscribe(
    request: JsonRpcRequest,
    state: &AppState,
    tx: Sender<Message>,
) -> Result<JoinHandle<()>> {
    let NewContent {
        new_token,
        new_buy,
        new_sell,
    } = NewContent::new(state).await;

    let message = NewContentMessage {
        new_token,
        new_buy,
        new_sell,
    };
    let message_json = serde_json::to_value(message).context("Failed to serialize coin")?;
    send_success_response(&tx, request.method(), json!(message_json)).await?;
    let mut receiver = state.new_content_producer.get_content_receiver().await;

    let handle = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            info!("New content message: {:?}", message);

            if let Err(e) = send_success_response(&tx, request.method(), json!(message)).await {
                error!("Failed to send new event: {:?}", e);
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

fn parse_chart(params: Option<&Value>) -> Option<String> {
    match params {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(obj)) => obj
            .get("chart")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned()),
        _ => None,
    }
}

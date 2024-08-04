use anyhow::{Context, Result};
use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;
#[derive(Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default = "default_jsonrpc")]
    jsonrpc: String,
    pub method: JsonRpcMethod,
    pub params: Option<Value>,
    id: u8,
}

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    Success {
        jsonrpc: String,
        method: JsonRpcMethod,
        result: Option<Value>,
    },
    Error {
        jsonrpc: String,
        error: JsonRpcError,
    },
}

#[derive(Serialize)]
pub struct JsonRpcError {
    code: JsonRpcErrorCode,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum JsonRpcErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerError = -32000,
    Unauthorized = -32001,
    NotFound = -32002,
    RateLimitExceeded = -32003,
}

impl JsonRpcErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            JsonRpcErrorCode::ParseError => "Parse error",
            JsonRpcErrorCode::InvalidRequest => "Invalid request",
            JsonRpcErrorCode::MethodNotFound => "Method not found",
            JsonRpcErrorCode::InvalidParams => "Invalid params",
            JsonRpcErrorCode::InternalError => "Internal error",
            JsonRpcErrorCode::ServerError => "Server error",
            JsonRpcErrorCode::Unauthorized => "Unauthorized",
            JsonRpcErrorCode::NotFound => "Not found",
            JsonRpcErrorCode::RateLimitExceeded => "Rate limit exceeded",
        }
    }
}

pub async fn send_success_response(
    tx: &Sender<Message>,
    method: &JsonRpcMethod,
    result: Value,
) -> Result<()> {
    let response = JsonRpcResponse::Success {
        jsonrpc: "2.0".to_string(),
        method: method.to_owned(),
        result: Some(result),
    };
    send_response(tx, response).await
}

pub async fn send_error_response(
    tx: &Sender<Message>,
    code: JsonRpcErrorCode,
    message: &str,
) -> Result<()> {
    let response = JsonRpcResponse::Error {
        jsonrpc: "2.0".to_string(),
        error: JsonRpcError {
            code,
            message: message.to_string(),
        },
    };
    send_response(tx, response).await
}

pub async fn send_response(tx: &Sender<Message>, response: JsonRpcResponse) -> Result<()> {
    tx.send(Message::Text(serde_json::to_string(&response)?))
        .await
        .context("Failed to send response")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonRpcMethod {
    OrderSubscribe,
    CoinSubscribe,
    // 다른 메서드들을 여기에 추가할 수 있습니다.
}

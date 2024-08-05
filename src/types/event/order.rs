use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::model::{Coin, CoinReplyCount, Curve, Swap};
use sqlx::FromRow;
#[derive(Debug, Clone, Serialize)]
pub enum OrderEvent {
    CreationTime(Coin),
    BumpOrder(Swap),
    ReplyChange(CoinReplyCount),
    MartKetCap(Curve),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    #[serde(rename = "creation_time")]
    CreationTime,
    #[serde(rename = "market_cap")]
    MarketCap,
    #[serde(rename = "bump")]
    Bump,
    #[serde(rename = "reply_count")]
    ReplyCount,
    #[serde(rename = "latest_reply")]
    LatestReply,
}

impl FromStr for OrderType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "creation_time" => Ok(OrderType::CreationTime),
            "market_cap" => Ok(OrderType::MarketCap),
            "bump" => Ok(OrderType::Bump),
            "reply_count" => Ok(OrderType::ReplyCount),
            "lastest_reply" => Ok(OrderType::LatestReply),
            _ => Err(anyhow::anyhow!("Invalid order type: {}", s)),
        }
    }
}

impl OrderType {}
use super::SendMessageType;

#[derive(Debug, Clone, Serialize)]
pub struct OrderMessage {
    pub message_type: SendMessageType,
    pub new_token: Option<CreateCoinMessage>,
    pub new_swap: Option<CreateSwapMesage>,
    pub order_type: OrderType,
    pub order_coin: Option<Vec<OrderCoinResponse>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCoinMessage {
    pub creator: User,
    pub symbol: String,
    pub image_uri: String,
    pub created_at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub nickname: String,
    pub image_uri: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct CreateSwapMesage {
    pub trader_info: User,
    pub coin_info: CreateSwapCoinInfo,
    pub is_buy: bool,
    pub nad_amount: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CreateSwapCoinInfo {
    pub symbol: String,
    pub image_uri: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderCoinResponse {
    pub id: String,          //coin.id
    pub creator: User,       //coin 의 creator -> account table -> select nickname, image uri
    pub name: String,        // coin.name
    pub symbol: String,      //coin.symbol
    pub image_uri: String,   // coin.image_uri
    pub description: String, //coin.description
    pub reply_count: String, //coin.id -> coin_reply_count table -> select count
    pub price: String,       // coin.id -> curve table -> select price
    pub created_at: i64,
}

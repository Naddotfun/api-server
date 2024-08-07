use std::str::FromStr;

use super::{NewSwapMessage, NewTokenMessage, SendMessageType, User};
use crate::types::model::{Coin, CoinReplyCount, Curve, Swap};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone, Serialize)]
pub struct OrderMessage {
    pub message_type: SendMessageType,
    pub new_token: Option<NewTokenMessage>,
    pub new_swap: Option<NewSwapMessage>,
    pub order_type: OrderType,
    pub order_token: Option<Vec<OrderTokenResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CreateSwapCoinInfo {
    pub symbol: String,
    pub image_uri: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderTokenResponse {
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

use std::str::FromStr;

use super::{NewSwapMessage, NewTokenMessage, SendMessageType, UserInfo};
use crate::types::model::{Curve, Swap, Token, TokenReplyCount};
use serde::{Deserialize, Serialize};

use sqlx::FromRow;
use utoipa::ToSchema;

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
            "latest_reply" => Ok(OrderType::LatestReply),
            _ => Err(anyhow::anyhow!("Invalid order type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderMessage {
    pub order_type: OrderType,
    pub order_token: Option<Vec<OrderTokenResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct OrderTokenResponse {
    pub id: String,          //token.id
    pub user_info: UserInfo, //token 의 creator -> account table -> select nickname, image uri
    pub name: String,        // token.name
    pub symbol: String,      //token.symbol
    pub image_uri: String,   // token.image_uri
    pub description: String, //token.description
    pub reply_count: String, //token.id -> token_reply_count table -> select count
    pub price: String,       // token.id -> curve table -> select price
    pub created_at: i64,
}

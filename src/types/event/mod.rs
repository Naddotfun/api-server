pub mod capture;
pub mod coin;
pub mod new_content;
pub mod order;
use order::OrderTokenResponse;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::model::{Coin, Swap};

#[derive(FromRow)]
pub struct CoinAndUserInfo {
    pub coin_id: String,
    pub coin_symbol: String,
    pub coin_image_uri: String,
    pub user_nickname: String,
    pub user_image_uri: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub nickname: String,
    pub image_uri: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CoinInfo {
    pub symbol: String,
    pub image_uri: String,
}
#[derive(Debug, Clone, Serialize)]
pub enum SendMessageType {
    ALL,
    Regular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTokenMessage {
    pub user_info: UserInfo,
    pub symbol: String,
    pub image_uri: String,
    pub created_at: i64,
}

impl NewTokenMessage {
    pub fn new(order_token: &OrderTokenResponse) -> Self {
        NewTokenMessage {
            user_info: order_token.user_info.clone(),
            symbol: order_token.symbol.clone(),
            image_uri: order_token.image_uri.clone(),
            created_at: order_token.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSwapMessage {
    pub trader_info: UserInfo,
    pub coin_info: CoinInfo,
    pub is_buy: bool,
    pub nad_amount: String,
}

impl NewSwapMessage {
    pub fn new(order_token: &OrderTokenResponse, trader_info: UserInfo, swap: &Swap) -> Self {
        NewSwapMessage {
            trader_info,
            coin_info: CoinInfo {
                symbol: order_token.symbol.clone(),
                image_uri: order_token.image_uri.clone(),
            },
            is_buy: swap.is_buy,
            nad_amount: swap.nad_amount.to_string(),
        }
    }
}

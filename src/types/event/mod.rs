pub mod coin_message;
pub mod order;
pub mod wrapper;

use order::{CreateSwapCoinInfo, OrderTokenResponse};
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
pub struct User {
    pub nickname: String,
    pub image_uri: String,
}
#[derive(Debug, Clone, Serialize)]
pub enum SendMessageType {
    ALL,
    Regular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTokenMessage {
    pub creator: User,
    pub symbol: String,
    pub image_uri: String,
    pub created_at: i64,
}

impl NewTokenMessage {
    pub fn new(order_token: &OrderTokenResponse) -> Self {
        NewTokenMessage {
            creator: order_token.creator.clone(),
            symbol: order_token.symbol.clone(),
            image_uri: order_token.image_uri.clone(),
            created_at: order_token.created_at.clone(),
        }
    }
}
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct NewBuyMessage {
//     pub trader_info: User,
//     pub coin_info: CreateSwapCoinInfo,
//     pub nad_amount: String,
// }
// impl NewBuyMessage {
//     pub fn new(order_token: &OrderTokenResponse, trader_info: User, swap: &Swap) -> Self {
//         NewBuyMessage {
//             trader_info,
//             coin_info: CreateSwapCoinInfo {
//                 symbol: order_token.symbol.clone(),
//                 image_uri: order_token.image_uri.clone(),
//             },
//             nad_amount: swap.nad_amount.to_string(),
//         }
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct NewSellMessage {
//     pub trader_info: User,
//     pub coin_info: CreateSwapCoinInfo,
//     pub nad_amount: String,
// }
// impl NewSellMessage {
//     pub fn new(order_token: &OrderTokenResponse, trader_info: User, swap: &Swap) -> Self {
//         NewSellMessage {
//             trader_info,
//             coin_info: CreateSwapCoinInfo {
//                 symbol: order_token.symbol.clone(),
//                 image_uri: order_token.image_uri.clone(),
//             },
//             nad_amount: swap.nad_amount.to_string(),
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSwapMessage {
    pub trader_info: User,
    pub coin_info: CreateSwapCoinInfo,
    pub is_buy: bool,
    pub nad_amount: String,
}

impl NewSwapMessage {
    pub fn new(order_token: &OrderTokenResponse, trader_info: User, swap: &Swap) -> Self {
        NewSwapMessage {
            trader_info,
            coin_info: CreateSwapCoinInfo {
                symbol: order_token.symbol.clone(),
                image_uri: order_token.image_uri.clone(),
            },
            is_buy: swap.is_buy,
            nad_amount: swap.nad_amount.to_string(),
        }
    }
}

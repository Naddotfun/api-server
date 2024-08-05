use serde::{Deserialize, Serialize};

use crate::types::model::{Balance, Chart, Coin, Curve, Swap, Thread};

use super::{
    order::{CreateSwapCoinInfo, User},
    CoinAndUserInfo, NewSwapMessage,
};
use super::{NewTokenMessage, SendMessageType};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CoinResponse {
    pub id: String,
    pub swap: Option<Vec<Swap>>,
    pub chart: Option<Vec<Chart>>,
    pub balance: Option<Vec<Balance>>,
    pub curve: Option<Curve>,
    pub thread: Option<Vec<Thread>>,
}
#[derive(Debug, Clone, Serialize)]
pub struct CoinMessage {
    pub message_type: SendMessageType,
    pub new_token: Option<NewTokenMessage>,
    pub new_swap: Option<NewSwapMessage>,
    pub coin: CoinResponse,
}

impl CoinMessage {
    pub fn from_coin(coin: Coin, info: CoinAndUserInfo) -> Self {
        CoinMessage {
            message_type: SendMessageType::ALL,
            new_token: Some(NewTokenMessage {
                creator: User {
                    nickname: info.user_nickname,
                    image_uri: info.user_image_uri,
                },
                symbol: info.coin_symbol,
                image_uri: info.coin_image_uri,
                created_at: coin.created_at,
            }),
            new_swap: None,
            coin: CoinResponse {
                id: coin.id.clone(),
                swap: None,
                chart: None,
                balance: None,
                curve: None,
                thread: None,
            },
        }
    }
    pub fn from_swap(swap: Swap, info: CoinAndUserInfo) -> Self {
        CoinMessage {
            message_type: SendMessageType::ALL,
            new_token: None,
            new_swap: Some(NewSwapMessage {
                trader_info: User {
                    nickname: info.user_nickname,
                    image_uri: info.user_image_uri,
                },
                coin_info: CreateSwapCoinInfo {
                    symbol: info.coin_symbol,
                    image_uri: info.coin_image_uri,
                },
                is_buy: swap.is_buy,
                nad_amount: swap.nad_amount.to_string(),
            }),
            coin: CoinResponse {
                id: swap.coin_id.clone(),
                swap: None,
                chart: None,
                balance: None,
                curve: None,
                thread: None,
            },
        }
    }
    pub fn from_chart(chart: Chart) -> Self {
        CoinMessage {
            message_type: SendMessageType::Regular,
            new_token: None,
            new_swap: None,
            coin: CoinResponse {
                id: chart.coin_id.clone(),
                swap: None,
                chart: Some(vec![chart]),
                balance: None,
                curve: None,
                thread: None,
            },
        }
    }
    pub fn from_balance(balance: Balance) -> Self {
        CoinMessage {
            message_type: SendMessageType::Regular,
            new_token: None,
            new_swap: None,
            coin: CoinResponse {
                id: balance.coin_id.clone(),
                swap: None,
                chart: None,
                balance: Some(vec![balance]),
                curve: None,
                thread: None,
            },
        }
    }
    pub fn from_curve(curve: Curve) -> Self {
        CoinMessage {
            message_type: SendMessageType::Regular,
            new_token: None,
            new_swap: None,
            coin: CoinResponse {
                id: curve.coin_id.clone(),
                swap: None,
                chart: None,
                balance: None,
                curve: Some(curve),
                thread: None,
            },
        }
    }

    pub fn from_thread(thread: Thread) -> Self {
        CoinMessage {
            message_type: SendMessageType::Regular,
            new_token: None,
            new_swap: None,
            coin: CoinResponse {
                id: thread.coin_id.clone(),
                swap: None,
                chart: None,
                balance: None,
                curve: None,
                thread: Some(vec![thread]),
            },
        }
    }
}

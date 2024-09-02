use serde::{Deserialize, Serialize};
use tracing::info;

use crate::types::model::{
    Balance, BalanceWrapper, Chart, ChartWrapper, Curve, Swap, Thread, ThreadWrapper, Token,
};

use super::order::OrderType;
use super::{NewSwapMessage, NewTokenMessage, SendMessageType, UserInfo};
use super::{TokenAndUserInfo, TokenInfo};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenResponse {
    pub id: String,
    #[serde(rename = "swaps")]
    pub swap: Option<Vec<Swap>>,
    #[serde(rename = "charts")]
    pub chart: Option<Vec<ChartWrapper>>,
    #[serde(rename = "balances")]
    pub balance: Option<Vec<BalanceWrapper>>,
    pub curve: Option<Curve>,

    #[serde(rename = "threads")]
    pub thread: Option<Vec<ThreadWrapper>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenMessage {
    // pub message_type: SendMessageType,
    // pub order_type: OrderType,
    // pub new_token: Option<NewTokenMessage>,
    // pub new_buy: Option<NewSwapMessage>,
    // pub new_sell: Option<NewSwapMessage>,
    pub token: TokenResponse,
}

impl TokenMessage {
    pub fn from_token(token: Token, info: TokenAndUserInfo) -> Self {
        TokenMessage {
            // new_token: Some(NewTokenMessage {
            //     user_info: UserInfo {
            //         nickname: info.user_nickname,
            //         image_uri: info.user_image_uri,
            //     },
            //     id: token.id.clone(),
            //     symbol: info.token_symbol,
            //     image_uri: info.token_image_uri,
            //     created_at: token.created_at,
            // }),
            // new_buy: None,
            // new_sell: None,
            token: TokenResponse {
                id: token.id.clone(),
                swap: None,
                chart: None,
                balance: None,
                curve: None,
                thread: None,
            },
        }
    }
    pub fn from_swap(swap: Swap, info: TokenAndUserInfo) -> Self {
        match swap.is_buy {
            true => TokenMessage {
                // message_type: SendMessageType::ALL,
                // new_token: None,
                // new_buy: Some(NewSwapMessage {
                //     user_info: UserInfo {
                //         nickname: info.user_nickname,
                //         image_uri: info.user_image_uri,
                //     },
                //     is_buy: true,
                //     token_info: TokenInfo {
                //         id: info.token_id,
                //         symbol: info.token_symbol,
                //         image_uri: info.token_image_uri,
                //     },
                //     nad_amount: swap.nad_amount.to_string(),
                // }),
                // new_sell: None,
                token: TokenResponse {
                    id: swap.token_id.clone(),
                    swap: Some(vec![swap]),
                    chart: None,
                    balance: None,
                    curve: None,
                    thread: None,
                },
            },
            false => TokenMessage {
                // message_type: SendMessageType::ALL,
                // new_token: None,
                // new_buy: None,
                // new_sell: Some(NewSwapMessage {
                //     user_info: UserInfo {
                //         nickname: info.user_nickname,
                //         image_uri: info.user_image_uri,
                //     },
                //     is_buy: false,
                //     token_info: TokenInfo {
                //         id: info.token_id,
                //         symbol: info.token_symbol,
                //         image_uri: info.token_image_uri,
                //     },
                //     nad_amount: swap.nad_amount.to_string(),
                // }),
                token: TokenResponse {
                    id: swap.token_id.clone(),
                    swap: Some(vec![swap]),
                    chart: None,
                    balance: None,
                    curve: None,
                    thread: None,
                },
            },
        }
    }
    pub fn from_chart(chart: ChartWrapper) -> Self {
        TokenMessage {
            // message_type: SendMessageType::Regular,
            // new_token: None,
            // new_buy: None,
            // new_sell: None,
            token: TokenResponse {
                id: chart.token_id.clone(),
                swap: None,
                chart: Some(vec![chart]),
                balance: None,
                curve: None,
                thread: None,
            },
        }
    }
    pub fn from_balance(balance: BalanceWrapper) -> Self {
        TokenMessage {
            // message_type: SendMessageType::Regular,
            // new_token: None,
            // new_buy: None,
            // new_sell: None,
            token: TokenResponse {
                id: balance.token_id.clone(),
                swap: None,
                chart: None,
                balance: Some(vec![balance]),
                curve: None,
                thread: None,
            },
        }
    }
    pub fn from_curve(curve: Curve) -> Self {
        TokenMessage {
            // message_type: SendMessageType::Regular,
            // new_token: None,
            // new_buy: None,
            // new_sell: None,
            token: TokenResponse {
                id: curve.token_id.clone(),
                swap: None,
                chart: None,
                balance: None,
                curve: Some(curve),
                thread: None,
            },
        }
    }

    pub fn from_thread(thread: ThreadWrapper) -> Self {
        TokenMessage {
            // message_type: SendMessageType::Regular,
            // new_token: None,
            // new_buy: None,
            // new_sell: None,
            token: TokenResponse {
                id: thread.token_id.clone(),
                swap: None,
                chart: None,
                balance: None,
                curve: None,
                thread: Some(vec![thread]),
            },
        }
    }
}

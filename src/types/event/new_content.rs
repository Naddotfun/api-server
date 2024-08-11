use serde::Serialize;

use crate::types::model::{Coin, Swap};

use super::{CoinAndUserInfo, CoinInfo, NewSwapMessage, NewTokenMessage, UserInfo};

#[derive(Debug, Clone, Serialize)]
pub struct NewContentMessage {
    pub new_buy: Option<NewSwapMessage>,
    pub new_sell: Option<NewSwapMessage>,
    pub new_token: Option<NewTokenMessage>,
}
impl NewContentMessage {
    pub fn from_coin(coin: Coin, info: CoinAndUserInfo) -> Self {
        NewContentMessage {
            new_token: Some(NewTokenMessage {
                user_info: UserInfo {
                    nickname: info.user_nickname,
                    image_uri: info.user_image_uri,
                },
                symbol: info.coin_symbol,
                image_uri: info.coin_image_uri,
                created_at: coin.created_at,
            }),
            new_buy: None,
            new_sell: None,
        }
    }

    pub fn from_swap(swap: Swap, info: CoinAndUserInfo) -> Self {
        match swap.is_buy {
            true => NewContentMessage {
                new_token: None,
                new_buy: Some(NewSwapMessage {
                    trader_info: UserInfo {
                        nickname: info.user_nickname,
                        image_uri: info.user_image_uri,
                    },
                    is_buy: true,
                    coin_info: CoinInfo {
                        symbol: info.coin_symbol,
                        image_uri: info.coin_image_uri,
                    },
                    nad_amount: swap.nad_amount.to_string(),
                }),
                new_sell: None,
            },
            false => NewContentMessage {
                new_token: None,
                new_buy: None,
                new_sell: Some(NewSwapMessage {
                    trader_info: UserInfo {
                        nickname: info.user_nickname,
                        image_uri: info.user_image_uri,
                    },
                    is_buy: false,
                    coin_info: CoinInfo {
                        symbol: info.coin_symbol,
                        image_uri: info.coin_image_uri,
                    },
                    nad_amount: swap.nad_amount.to_string(),
                }),
            },
        }
    }
}

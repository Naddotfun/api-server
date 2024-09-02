use serde::Serialize;

use crate::types::model::{Swap, Token};

use super::{NewSwapMessage, NewTokenMessage, TokenAndUserInfo, TokenInfo, UserInfo};

#[derive(Debug, Clone, Serialize)]
pub struct NewContentMessage {
    pub new_buy: Option<NewSwapMessage>,
    pub new_sell: Option<NewSwapMessage>,
    pub new_token: Option<NewTokenMessage>,
}
impl NewContentMessage {
    pub fn from_token(token: Token, info: TokenAndUserInfo) -> Self {
        NewContentMessage {
            new_token: Some(NewTokenMessage {
                user_info: UserInfo {
                    nickname: info.user_nickname,
                    image_uri: info.user_image_uri,
                },
                id: token.id.clone(),
                symbol: info.token_symbol,
                image_uri: info.token_image_uri,
                created_at: token.created_at,
            }),
            new_buy: None,
            new_sell: None,
        }
    }

    pub fn from_swap(swap: Swap, info: TokenAndUserInfo) -> Self {
        match swap.is_buy {
            true => NewContentMessage {
                new_token: None,
                new_buy: Some(NewSwapMessage {
                    user_info: UserInfo {
                        nickname: info.user_nickname,
                        image_uri: info.user_image_uri,
                    },
                    is_buy: true,
                    token_info: TokenInfo {
                        id: info.token_id,
                        symbol: info.token_symbol,
                        image_uri: info.token_image_uri,
                    },
                    nad_amount: swap.nad_amount.to_string(),
                }),
                new_sell: None,
            },
            false => NewContentMessage {
                new_token: None,
                new_buy: None,
                new_sell: Some(NewSwapMessage {
                    user_info: UserInfo {
                        nickname: info.user_nickname,
                        image_uri: info.user_image_uri,
                    },
                    is_buy: false,
                    token_info: TokenInfo {
                        id: info.token_id,
                        symbol: info.token_symbol,
                        image_uri: info.token_image_uri,
                    },
                    nad_amount: swap.nad_amount.to_string(),
                }),
            },
        }
    }
}

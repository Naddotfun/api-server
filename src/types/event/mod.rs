pub mod capture;
pub mod new_content;
pub mod order;
pub mod token;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use super::model::{Swap, Token};

#[derive(FromRow)]
pub struct TokenAndUserInfo {
    pub token_id: String,
    pub token_symbol: String,
    pub token_image_uri: String,
    pub user_nickname: String,
    pub user_image_uri: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    pub nickname: String,
    pub image_uri: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TokenInfo {
    pub id: String,
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
    pub id: String,
    pub symbol: String,
    pub image_uri: String,
    pub created_at: i64,
}

impl NewTokenMessage {
    pub fn new(token: &Token, token_and_user_info: &TokenAndUserInfo) -> Self {
        NewTokenMessage {
            user_info: UserInfo {
                nickname: token_and_user_info.user_nickname.clone(),
                image_uri: token_and_user_info.user_image_uri.clone(),
            },
            id: token_and_user_info.token_id.clone(),
            symbol: token_and_user_info.token_symbol.clone(),
            image_uri: token_and_user_info.token_image_uri.clone(),
            created_at: token.created_at,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSwapMessage {
    pub user_info: UserInfo,
    pub token_info: TokenInfo,
    pub is_buy: bool,
    pub nad_amount: String,
}

impl NewSwapMessage {
    pub fn new(swap: &Swap, token_and_user_info: &TokenAndUserInfo) -> Self {
        NewSwapMessage {
            user_info: UserInfo {
                nickname: token_and_user_info.user_nickname.clone(),
                image_uri: token_and_user_info.user_image_uri.clone(),
            },
            token_info: TokenInfo {
                id: token_and_user_info.token_id.clone(),
                symbol: token_and_user_info.token_symbol.clone(),
                image_uri: token_and_user_info.token_image_uri.clone(),
            },
            is_buy: swap.is_buy,
            nad_amount: swap.nad_amount.to_string(),
        }
    }
}

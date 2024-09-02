use bigdecimal::BigDecimal;
use serde::Serialize;
use utoipa::ToSchema;

use super::model::Token;

pub enum Identifier {
    Nickname(String),
    Address(String), // 이더리움 주소
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HoldToken {
    pub token: Token,
    pub balance: String,
    pub price: String,
}

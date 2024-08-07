use bigdecimal::BigDecimal;
use serde::Serialize;
use utoipa::ToSchema;

use super::model::Coin;

pub enum Identifier {
    Nickname(String),
    Address(String), // 이더리움 주소
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HoldCoin {
    pub coin: Coin,
    pub balance: BigDecimal,
    pub price: BigDecimal,
}

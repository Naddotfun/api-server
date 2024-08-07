use bigdecimal::BigDecimal;
use model::Coin;
use serde::Serialize;

pub mod event;
pub mod model;

#[derive(Debug, Clone, Serialize)]
pub struct HoldCoinResponse {
    pub coin: Coin,
    pub balance: BigDecimal,
    pub price: BigDecimal,
}

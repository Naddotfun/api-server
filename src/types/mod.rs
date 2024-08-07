use bigdecimal::BigDecimal;
use model::Coin;

use serde::Serialize;

pub mod event;
pub mod model;
pub mod profile;
#[derive(Debug, Clone, Serialize)]
pub struct HoldCoinResponse {
    pub coin: Coin,
    pub balance: BigDecimal,
    pub price: BigDecimal,
}

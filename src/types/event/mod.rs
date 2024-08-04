use serde::Serialize;
use sqlx::FromRow;

pub mod coin;
pub mod order;
pub mod wrapper;

#[derive(FromRow)]
pub struct CoinAndUserInfo {
    pub coin_id: String,
    pub coin_symbol: String,
    pub coin_image_uri: String,
    pub user_nickname: String,
    pub user_image_uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum SendMessageType {
    ALL,
    Regular,
}

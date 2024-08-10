// use bigDecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::types::BigDecimal;

use tracing::info;
use utoipa::ToSchema;

fn serialize_bigdecimal<S>(x: &BigDecimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    info!("x = {:?}", x.to_string());

    let rounded = x.round(9).with_scale(10);
    info!("rounded = {:?}", rounded.to_string());

    let format = rounded.to_f64().unwrap_or(0.0).to_string();
    info!("f64_val: {}", format);

    s.serialize_str(&format)
}
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Coin {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub creator: String,
    pub description: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub image_uri: String,
    pub is_listing: bool,
    pub created_at: i64,
    pub create_transaction_hash: String,
    pub is_updated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Curve {
    pub id: String,
    pub coin_id: String,

    pub virtual_nad: BigDecimal,

    pub virtual_token: BigDecimal,
    pub latest_trade_at: i64,
    #[serde(serialize_with = "serialize_bigdecimal")]
    pub price: BigDecimal,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Swap {
    pub id: i32,
    pub coin_id: String,
    pub sender: String,
    pub is_buy: bool,
    pub nad_amount: BigDecimal,
    pub token_amount: BigDecimal,
    pub created_at: i64,
    pub transaction_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Chart {
    //id = 읽어오기 위해서 임의로 생성
    pub id: i32,
    pub coin_id: String,
    #[serde(serialize_with = "serialize_bigdecimal")]
    pub open_price: BigDecimal,
    #[serde(serialize_with = "serialize_bigdecimal")]
    pub close_price: BigDecimal,
    #[serde(serialize_with = "serialize_bigdecimal")]
    pub high_price: BigDecimal,
    #[serde(serialize_with = "serialize_bigdecimal")]
    pub low_price: BigDecimal,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Account {
    pub id: String,
    pub image_uri: String,
    pub nickname: String,
    pub bio: String,
    pub follower_count: i32,
    pub following_count: i32,
    pub like_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Balance {
    pub id: i32,
    pub coin_id: String,
    pub account: String,
    pub amount: BigDecimal,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountSession {
    pub id: String,         //session_id
    pub account_id: String, //account_id
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow, ToSchema)]
pub struct Thread {
    pub id: i32,

    pub coin_id: String,

    pub author_id: String,

    pub content: String,

    #[schema(value_type = String, example = "2023-06-01T12:00:00Z")]
    pub created_at: DateTime<Utc>,

    #[schema(value_type = String, example = "2023-06-01T12:00:00Z")]
    pub updated_at: DateTime<Utc>,

    pub root_id: Option<i32>,

    pub likes_count: i32,

    pub reply_count: i32,

    pub image_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct CoinReplyCount {
    pub coin_id: String,
    pub reply_count: i32,
}

use anyhow::{Context, Result};
use bigdecimal::{BigDecimal, RoundingMode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use sqlx;
use utoipa::ToSchema;

// Helper function for BigDecimal serialization
fn serialize_price_bigdecimal<S>(x: &BigDecimal, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let rounded = x.with_scale_round(11, RoundingMode::HalfUp);
    let str_val = rounded.to_string();
    let final_val = if str_val.contains('E') {
        let parts: Vec<&str> = str_val.split('E').collect();
        let base = parts[0].parse::<f64>().unwrap();
        let exp: i32 = parts[1].parse().unwrap();
        format!("{:.10}", base * 10f64.powi(exp))
    } else {
        format!("{:.10}", str_val.parse::<f64>().unwrap())
    };

    let trimmed = final_val.trim_end_matches('0').trim_end_matches('.');
    let result = if trimmed.is_empty() || trimmed == "0" {
        "0.0".to_string()
    } else if !trimmed.contains('.') {
        format!("{}.0", trimmed)
    } else {
        trimmed.to_string()
    };

    s.serialize_str(&result)
}

// Define structs
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
    #[serde(serialize_with = "serialize_price_bigdecimal")]
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
    pub id: i32,
    pub coin_id: String,
    #[serde(serialize_with = "serialize_price_bigdecimal")]
    pub open_price: BigDecimal,
    #[serde(serialize_with = "serialize_price_bigdecimal")]
    pub close_price: BigDecimal,
    #[serde(serialize_with = "serialize_price_bigdecimal")]
    pub high_price: BigDecimal,
    #[serde(serialize_with = "serialize_price_bigdecimal")]
    pub low_price: BigDecimal,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Balance {
    pub id: i32,
    pub coin_id: String,
    pub account: String,
    pub amount: BigDecimal,
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountSession {
    pub id: String,
    pub account_id: String,
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

// Wrapper structs
#[derive(Debug, Deserialize)]
pub struct CoinWrapper {
    record: Coin,

    coin_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CurveWrapper {
    record: Curve,
    #[serde(skip_serializing)]
    coin_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SwapWrapper {
    record: Swap,
    #[serde(skip_serializing)]
    coin_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BalanceWrapper {
    record: Balance,
    #[serde(skip_serializing)]
    coin_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ThreadWrapper {
    record: Thread,
    #[serde(skip_serializing)]
    coin_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CoinReplyCountWrapper {
    record: CoinReplyCount,
    #[serde(skip_serializing)]
    coin_id: String,
}

// ChartWrapper implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartWrapper {
    pub record: Chart,
    #[serde(skip_serializing)]
    pub chart_type: String,
    #[serde(skip_serializing)]
    pub coin_id: String,
}

impl ChartWrapper {
    pub fn from_value(value: Value) -> Result<ChartWrapper> {
        serde_json::from_value(value).context("Failed to deserialize ChartWrapper")
    }
}

// Macro for implementing FromValue trait
macro_rules! impl_from_value {
    ($type:ty, $wrapper:ty) => {
        impl FromValue for $type {
            fn from_value(value: Value) -> Result<Self> {
                let wrapper = serde_json::from_value::<$wrapper>(value)
                    .context(concat!("Failed to deserialize ", stringify!($type)))?;
                Ok(wrapper.record)
            }
        }
    };
}
pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self>;
}
// Implement FromValue for all types
impl_from_value!(Coin, CoinWrapper);
impl_from_value!(Curve, CurveWrapper);
impl_from_value!(Swap, SwapWrapper);
impl_from_value!(Balance, BalanceWrapper);
impl_from_value!(Thread, ThreadWrapper);
impl_from_value!(CoinReplyCount, CoinReplyCountWrapper);

use anyhow::{Context, Result};
use serde_json::Value;
use sqlx::{postgres::PgRow, FromRow, Row};
use std::sync::Arc;

use crate::{
    db::postgres::PostgresDatabase,
    types::{
        event::coin_message::{CoinMessage, CoinResponse},
        model::{Balance, Chart, Coin, Curve, Swap, Thread},
    },
};
#[derive(sqlx::FromRow)]
struct CoinResponseRaw {
    swap: Option<Value>,
    chart: Option<Value>,
    balance: Option<Value>,
    curve: Option<Value>,
    thread: Option<Value>,
}
pub struct CoinPageController {
    pub db: Arc<PostgresDatabase>,
}

impl CoinPageController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        CoinPageController { db }
    }

    pub async fn get_coin_message(&self, coin_id: &str) -> Result<CoinResponse> {
        let raw = sqlx::query_as::<_, CoinResponseRaw>(
            r#"
            SELECT 
                (SELECT json_agg(row_to_json(s)) FROM swaps s WHERE s.coin_id = $1) as swap,
                (SELECT json_agg(row_to_json(ch)) FROM charts ch WHERE ch.coin_id = $1) as chart,
                (SELECT json_agg(row_to_json(b)) FROM balances b WHERE b.coin_id = $1) as balance,
                (SELECT row_to_json(cu) FROM curves cu WHERE cu.coin_id = $1 LIMIT 1) as curve,
                (SELECT json_agg(row_to_json(t)) FROM threads t WHERE t.coin_id = $1) as thread
            "#,
        )
        .bind(coin_id)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(CoinResponse {
            id: coin_id.to_string(),
            swap: raw.swap.and_then(|v| serde_json::from_value(v).ok()),
            chart: raw.chart.and_then(|v| serde_json::from_value(v).ok()),
            balance: raw.balance.and_then(|v| serde_json::from_value(v).ok()),
            curve: raw.curve.and_then(|v| serde_json::from_value(v).ok()),
            thread: raw.thread.and_then(|v| serde_json::from_value(v).ok()),
        })
    }
}
// fn json_to_vec<T: serde::de::DeserializeOwned>(row: &PgRow, field: &str) -> Result<Option<Vec<T>>> {
//     let json: Value = row.try_get(field)?;
//     if json.is_null() || json.as_array().map_or(true, |arr| arr.is_empty()) {
//         Ok(None)
//     } else {
//         Ok(Some(
//             serde_json::from_value(json).context("Failed to deserialize JSON to Vec")?,
//         ))
//     }
// }

// fn json_to_option<T: serde::de::DeserializeOwned>(row: &PgRow, field: &str) -> Result<Option<T>> {
//     let json: Value = row.try_get(field)?;
//     if json.is_null() {
//         Ok(None)
//     } else {
//         Ok(Some(
//             serde_json::from_value(json).context("Failed to deserialize JSON to type")?,
//         ))
//     }
// }

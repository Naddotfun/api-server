use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::Arc;

use crate::{
    db::postgres::PostgresDatabase,
    types::{chart_type::ChartType, event::coin_message::CoinResponse},
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

    pub async fn get_coin_message(
        &self,
        coin_id: &str,
        chart_type: ChartType,
    ) -> Result<CoinResponse> {
        let chart_table = match chart_type {
            ChartType::OneMinute => "chart_1m",
            ChartType::FiveMinutes => "chart_5m",
            ChartType::FifteenMinutes => "chart_15m",
            ChartType::ThirtyMinutes => "chart_30m",
            ChartType::OneHour => "chart_1h",
            ChartType::FourHours => "chart_4h",
            ChartType::OneDay => "chart_1d",
        };

        let query = format!(
            r#"
            SELECT 
                (SELECT json_agg(row_to_json(s)) FROM swap s WHERE s.coin_id = $1) as swap,
                (SELECT json_agg(row_to_json(ch)) FROM {} ch WHERE ch.coin_id = $1) as chart,
                (SELECT json_agg(row_to_json(b)) FROM balance b WHERE b.coin_id = $1) as balance,
                (SELECT row_to_json(cu) FROM curve cu WHERE cu.coin_id = $1 LIMIT 1) as curve,
                (SELECT json_agg(row_to_json(t)) FROM thread t WHERE t.coin_id = $1) as thread
            "#,
            chart_table
        );

        let raw = sqlx::query_as::<_, CoinResponseRaw>(&query)
            .bind(coin_id)
            .fetch_one(&self.db.pool)
            .await
            .context("Failed to fetch coin data")?;

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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::types::model::{Balance, Chart, Coin, CoinReplyCount, Curve, Swap, Thread};
use anyhow::{Context, Result};
#[derive(Debug, Deserialize)]
pub struct CoinWrapper {
    pub record: Coin,
    pub coin_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartWrapper {
    pub record: Chart,

    pub chart_type: String,

    pub coin_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceWrapper {
    pub record: Balance,
    pub operation: String,
    pub coin_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveWrapper {
    pub operation: String,
    pub record: Curve,
    pub coin_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SwapWrapper {
    pub record: Swap,
    pub coin_id: String,
}
#[derive(Debug, Deserialize)]
pub struct ReplyCountWrapper {
    pub coin_id: String,
    pub record: CoinReplyCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadWrapper {
    pub record: Thread,
    pub operation: String,
    pub coin_id: String,
}

impl CoinWrapper {
    pub fn parse(value: Value) -> Result<Coin> {
        let coin_wrapper = serde_json::from_value::<CoinWrapper>(value)
            .context("Fail Deserialize coin wrapper")?;
        Ok(coin_wrapper.record)
    }
}

impl SwapWrapper {
    pub fn parse(value: Value) -> Result<Swap> {
        let swap_wrapper = serde_json::from_value::<SwapWrapper>(value)
            .context("Fail Deserialize swap wrapper")?;
        Ok(swap_wrapper.record)
    }
}

impl CurveWrapper {
    pub fn parse(value: Value) -> Result<Curve> {
        let curve_wrapper =
            serde_json::from_value::<CurveWrapper>(value).context("Fail Deserialize curve")?;
        Ok(curve_wrapper.record)
    }
}

impl ReplyCountWrapper {
    pub fn parse(value: Value) -> Result<CoinReplyCount> {
        let reply_wrapper = serde_json::from_value::<ReplyCountWrapper>(value)
            .context("Fail Deserialize reply wrapper")?;
        Ok(reply_wrapper.record)
    }
}

impl ChartWrapper {
    pub fn parse(value: Value) -> Result<ChartWrapper> {
        let chart =
            serde_json::from_value::<ChartWrapper>(value).context("Fail Deserialize chart")?;
        Ok(chart)
    }
}

impl BalanceWrapper {
    pub fn parse(value: Value) -> Result<Balance> {
        let balance =
            serde_json::from_value::<BalanceWrapper>(value).context("Fail Deserialize balance")?;
        Ok(balance.record)
    }
}

impl ThreadWrapper {
    pub fn parse(value: Value) -> Result<Thread> {
        let thread =
            serde_json::from_value::<ThreadWrapper>(value).context("Fail Deserialize thread")?;
        Ok(thread.record)
    }
}

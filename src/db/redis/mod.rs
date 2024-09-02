use crate::types::{
    event::{
        order::{OrderTokenResponse, OrderType},
        NewSwapMessage, NewTokenMessage,
    },
    model::Token,
};
use anyhow::{Context, Result};
use axum_extra::handler::Or;
use chrono::Utc;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use redis::{AsyncCommands, Client, Commands};
use serde_json::{from_str, Value};
use tracing::{error, info, warn};

use super::postgres::controller::order::TokenWithScore;
lazy_static! {
    static ref BUMP_ORDER_KEY: &'static str = "bump_order";
    static ref LAST_REPLY_ORDER_KEY: &'static str = "last_reply_order";
    static ref REPLY_COUNT_ORDER_KEY: &'static str = "reply_count_order";
    static ref MARKET_CAP_ORDER_KEY: &'static str = "market_cap_order";
    static ref CREATION_TIME_ORDER_KEY: &'static str = "creation_time_order";
    static ref NEW_TOKEN_KEY: &'static str = "new_token";
    static ref NEW_BUY_KEY: &'static str = "new_buy";
    static ref NEW_SELL_KEY: &'static str = "new_sell";
}
pub struct RedisDatabase {
    pub client: Client,
}

impl RedisDatabase {
    pub async fn new() -> Self {
        let client = {
            let host = std::env::var("REDIS_HOST").unwrap();
            let port = std::env::var("REDIS_PORT").unwrap();
            let connection_string = format!("redis://{}:{}", host, port);
            info!("Connecting to standalone Redis at: {}", connection_string);
            let client = Client::open(connection_string).unwrap();
            client
        };

        RedisDatabase { client }
    }
    // 범용 메서드: 정렬된 세트에서 코인 저장
    async fn add_token_to_queue(
        &self,
        key: &str,
        token: &OrderTokenResponse,
        score: String,
    ) -> Result<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let token_json = serde_json::to_string(token).context("Failed to serialize token")?;

        let score = score
            .parse::<f64>()
            .context("Failed to parse score as f64")?;

        let mut pipe = redis::pipe();
        pipe.atomic();

        // ZADD will update if the token exists, or insert if it doesn't
        pipe.zadd(key, &token_json, score);

        // Keep only the top 50 items (remove items from index 50 to the end)
        pipe.zremrangebyrank(key, 0, -51);

        // Get the rank of the token after the operation
        pipe.zrevrank(key, &token_json);

        let results: (i32, i32, Option<isize>) = pipe.query_async(&mut conn).await?;

        let added_or_updated = results.0;
        let removed = results.1;
        let rank = results.2;

        if added_or_updated == 1 {
            info!("Token {} was newly added to the queue", token.id);
        } else {
            info!("Token {} was updated in the queue", token.id);
        }

        if removed > 0 {
            info!("Removed {} item(s) to maintain the 50-item limit", removed);
        }

        if let Some(r) = rank {
            info!("Token {} is now at rank {}", token.id, r);
            Ok(r < 50)
        } else {
            info!("Token {} was not added/updated in the top 50", token.id);
            Ok(false)
        }
    }
    pub async fn add_to_bump_order(
        &self,
        token: &OrderTokenResponse,
        score: String,
    ) -> Result<bool> {
        self.add_token_to_queue(*BUMP_ORDER_KEY, token, score).await
    }

    pub async fn add_to_last_reply_order(
        &self,
        token: &OrderTokenResponse,
        score: String,
    ) -> Result<bool> {
        self.add_token_to_queue(*LAST_REPLY_ORDER_KEY, token, score)
            .await
    }

    pub async fn add_to_reply_count_order(
        &self,
        token: &OrderTokenResponse,
        score: String,
    ) -> Result<bool> {
        self.add_token_to_queue(*REPLY_COUNT_ORDER_KEY, token, score)
            .await
    }

    pub async fn add_to_market_cap_order(
        &self,
        token: &OrderTokenResponse,
        score: String,
    ) -> Result<bool> {
        self.add_token_to_queue(*MARKET_CAP_ORDER_KEY, token, score)
            .await
    }

    pub async fn add_to_creation_time_order(
        &self,
        token: &OrderTokenResponse,
        score: String,
    ) -> Result<bool> {
        self.add_token_to_queue(*CREATION_TIME_ORDER_KEY, token, score)
            .await
    }

    async fn get_tokens_from_queue(&self, key: &str) -> Result<Vec<OrderTokenResponse>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        // ZREVRANGE를 사용하여 모든 항목을 한 번에 가져옵니다 (최대 50개).
        let tokens_json: Vec<String> = conn.zrevrange(key, 0, -1).await?;

        // 병렬로 JSON 파싱
        let parsed_tokens_futures = tokens_json.into_iter().map(|json| {
            tokio::spawn(async move {
                match from_str(&json) {
                    Ok(token) => Ok(token),
                    Err(e) => {
                        error!("Failed to parse token JSON: {}. Error: {}", json, e);
                        Err(anyhow::anyhow!("JSON parsing error"))
                    }
                }
            })
        });

        let results = try_join_all(parsed_tokens_futures).await?;

        // 성공적으로 파싱된 코인만 수집
        let tokens: Vec<OrderTokenResponse> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();

        Ok(tokens)
    }
    pub async fn get_order(&self, order_type: OrderType) -> Result<Vec<OrderTokenResponse>> {
        use OrderType::*;
        match order_type {
            CreationTime => self.get_creation_time_order().await,
            Bump => self.get_bump_order().await,
            LatestReply => self.get_last_reply_order().await,
            ReplyCount => self.get_reply_count_order().await,
            MarketCap => self.get_market_cap_order().await,
        }
    }
    // 각 순서별로 모든 코인 가져오기
    pub async fn get_bump_order(&self) -> Result<Vec<OrderTokenResponse>> {
        self.get_tokens_from_queue(*BUMP_ORDER_KEY).await
    }

    pub async fn get_last_reply_order(&self) -> Result<Vec<OrderTokenResponse>> {
        self.get_tokens_from_queue(*LAST_REPLY_ORDER_KEY).await
    }

    pub async fn get_reply_count_order(&self) -> Result<Vec<OrderTokenResponse>> {
        self.get_tokens_from_queue(*REPLY_COUNT_ORDER_KEY).await
    }

    pub async fn get_market_cap_order(&self) -> Result<Vec<OrderTokenResponse>> {
        self.get_tokens_from_queue(*MARKET_CAP_ORDER_KEY).await
    }

    pub async fn get_creation_time_order(&self) -> Result<Vec<OrderTokenResponse>> {
        self.get_tokens_from_queue(*CREATION_TIME_ORDER_KEY).await
    }

    //Initialize

    async fn set_tokens_to_queue(&self, key: &str, tokens: Vec<TokenWithScore>) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let mut pipe = redis::pipe();

        // 기존 데이터 삭제
        pipe.del(key);

        if tokens.is_empty() {
            return Ok(());
        }

        // 코인 데이터를 JSON으로 직렬화하고 정렬된 집합에 추가
        for token_with_score in tokens {
            let token_json = serde_json::to_string(&token_with_score.token)
                .context("Failed to serialize token")?;
            let score = token_with_score
                .score
                .parse::<f64>()
                .context("Failed to parse score as f64")?;
            // Use the negative of the score to reverse the order

            pipe.zadd(key, token_json, score);
        }
        // 파이프라인 실행
        pipe.query_async(&mut conn)
            .await
            .context("Failed to execute Redis pipeline")?;

        Ok(())
    }
    pub async fn set_creation_time_order(
        &self,
        tokens_with_score: Vec<TokenWithScore>,
    ) -> Result<()> {
        self.set_tokens_to_queue(*CREATION_TIME_ORDER_KEY, tokens_with_score)
            .await
    }
    pub async fn set_bump_order(&self, tokens_with_score: Vec<TokenWithScore>) -> Result<()> {
        self.set_tokens_to_queue(*BUMP_ORDER_KEY, tokens_with_score)
            .await
    }

    pub async fn set_last_reply_order(&self, tokens_with_score: Vec<TokenWithScore>) -> Result<()> {
        self.set_tokens_to_queue(*LAST_REPLY_ORDER_KEY, tokens_with_score)
            .await
    }

    pub async fn set_reply_count_order(
        &self,
        tokens_with_score: Vec<TokenWithScore>,
    ) -> Result<()> {
        self.set_tokens_to_queue(*REPLY_COUNT_ORDER_KEY, tokens_with_score)
            .await
    }

    pub async fn set_market_cap_order(&self, tokens_with_score: Vec<TokenWithScore>) -> Result<()> {
        self.set_tokens_to_queue(*MARKET_CAP_ORDER_KEY, tokens_with_score)
            .await
    }

    pub async fn set_new_token(
        &self,
        new_token: &NewTokenMessage,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value = serde_json::to_string(new_token).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Serialization error",
                e.to_string(),
            ))
        })?;
        info!("Setting new token: {:?}", value);
        conn.set::<_, _, ()>(*NEW_TOKEN_KEY, value).await?;
        Ok(())
    }

    pub async fn get_new_token(&self) -> Result<Option<NewTokenMessage>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(*NEW_TOKEN_KEY).await?;
        match value {
            Some(v) => {
                let new_token = serde_json::from_str(&v).context("Failed to parse new token")?;
                Ok(Some(new_token))
            }
            None => Ok(None),
        }
    }

    pub async fn set_new_swap(&self, new_swap: &NewSwapMessage) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value = serde_json::to_string(new_swap).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Serialization error",
                e.to_string(),
            ))
        })?;
        info!("Setting new swap: {:?}", value);
        match new_swap.is_buy {
            true => conn.set::<_, _, ()>(*NEW_BUY_KEY, value).await?,
            false => conn.set::<_, _, ()>(*NEW_SELL_KEY, value).await?,
        }

        Ok(())
    }

    pub async fn get_new_buy(&self) -> Result<Option<NewSwapMessage>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(*NEW_BUY_KEY).await?;
        match value {
            Some(v) => {
                let new_swap = serde_json::from_str(&v).context("Failed to parse new swap")?;
                Ok(Some(new_swap))
            }
            None => Ok(None),
        }
    }

    pub async fn get_new_sell(&self) -> Result<Option<NewSwapMessage>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(*NEW_SELL_KEY).await?;
        match value {
            Some(v) => {
                let new_swap = serde_json::from_str(&v).context("Failed to parse new swap")?;
                Ok(Some(new_swap))
            }
            None => Ok(None),
        }
    }
}

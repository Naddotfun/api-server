use anyhow::{Context, Result};
use sqlx::FromRow;
use std::sync::Arc;
use tracing::info;

use crate::{
    db::postgres::PostgresDatabase,
    types::{
        event::{
            order::{OrderTokenResponse, OrderType},
            User,
        },
        model::Coin,
    },
};

pub struct OrderController {
    pub db: Arc<PostgresDatabase>,
}

const ORDER_LIMIT: i64 = 50;
#[derive(Debug, FromRow)]
pub struct CoinWithScore {
    #[sqlx(flatten)]
    pub coin: OrderTokenResponse,
    pub score: String,
}
#[derive(Debug, Clone, sqlx::FromRow)]
struct OrderTokenResponseRaw {
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub image_uri: Option<String>,
    pub description: Option<String>,
    pub reply_count: Option<String>,
    pub price: Option<String>,
    pub creator: serde_json::Value,
    pub created_at: Option<i64>,
}
#[derive(Debug, Clone, FromRow)]
struct IdScore {
    id: String,
    score: Option<String>,
}
impl OrderController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        OrderController { db }
    }
    pub async fn get_order_token_response_by_coin_id(
        &self,
        coin_id: &str,
    ) -> Result<OrderTokenResponse> {
        let coin_response: OrderTokenResponseRaw = sqlx::query_as(
            r#"
            SELECT 
                c.id,
                c.name,
                c.symbol,
                c.image_uri,
                c.description,
                c.created_at,
                COALESCE(crc.reply_count::TEXT, '0') as reply_count,
                COALESCE(cu.price::TEXT, '0') as price,
                json_build_object(
                    'nickname', a.nickname,
                    'image_uri', a.image_uri
                ) as creator
            FROM coin c
            LEFT JOIN account a ON c.creator = a.id
            LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
            LEFT JOIN curve cu ON c.id = cu.coin_id
            WHERE c.id = $1
            "#,
        )
        .bind(coin_id)
        .fetch_one(&self.db.pool)
        .await
        .context("Failed to fetch coin information")?;

        let creator: User = serde_json::from_value(coin_response.creator)
            .context("Failed to deserialize creator information")?;

        Ok(OrderTokenResponse {
            id: coin_response.id.unwrap_or_default(),
            name: coin_response.name.unwrap_or_default(),
            symbol: coin_response.symbol.unwrap_or_default(),
            image_uri: coin_response.image_uri.unwrap_or_default(),
            description: coin_response.description.unwrap_or_default(),
            reply_count: coin_response.reply_count.unwrap_or_default(),
            price: coin_response.price.unwrap_or_default(),
            creator,
            created_at: coin_response.created_at.unwrap_or_default(),
        })
    }
    async fn get_ordered_tokens(&self, order_type: OrderType) -> Result<Vec<CoinWithScore>> {
        let id_scores_query = match order_type {
            OrderType::CreationTime => {
                r#"
                SELECT id, created_at::TEXT as score 
                FROM coin 
                ORDER BY created_at
                DESC
                LIMIT $1
                "#
            }

            OrderType::MarketCap => {
                r#"
                SELECT coin_id as id,price::TEXT as score 
                FROM curve
                ORDER BY price DESC
                LIMIT $1
                "#
            }

            OrderType::ReplyCount => {
                r#"
                SELECT coin_id as id,reply_count::TEXT as score 
                FROM coin_reply_count
                ORDER BY reply_count DESC
                LIMIT $1
                "#
            }
            OrderType::LatestReply => {
                r#"
                SELECT DISTINCT ON (coin_id) coin_id as id, created_at::TEXT as score 
                FROM thread
                ORDER BY coin_id,created_at DESC 
                LIMIT $1
                "#
            }
            OrderType::Bump => {
                r#"
                SELECT DISTINCT ON (coin_id) coin_id as id, created_at::TEXT as score 
                FROM swap ORDER BY coin_id, created_at DESC 
                LIMIT $1
                "#
            }
        };
        let coin_responses_query = r#"
        SELECT c.id, c.name, c.symbol, c.image_uri, c.description,c.created_at, COALESCE(crc.reply_count::TEXT, '0') as reply_count, COALESCE(cu.price::TEXT, '0') as price, json_build_object('nickname', a.nickname, 'image_uri', a.image_uri) as creator
        FROM coin c
        LEFT JOIN account a ON c.creator = a.id 
        LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
        LEFT JOIN curve cu ON c.id = cu.coin_id 
        WHERE c.id = ANY($1)"#;
        let id_scores: Vec<IdScore> = sqlx::query_as(id_scores_query)
            .bind(ORDER_LIMIT)
            .fetch_all(&self.db.pool)
            .await
            .context("Failed to fetch sorted coin IDs and scores")?;

        let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

        let coin_responses: Vec<OrderTokenResponseRaw> = sqlx::query_as(coin_responses_query)
            .bind(&ids[..])
            .fetch_all(&self.db.pool)
            .await
            .context("Failed to fetch detailed coin information")?;

        let coins_with_score = id_scores
            .into_iter()
            .filter_map(|id_score| {
                let coin_raw = coin_responses
                    .iter()
                    .find(|c| c.id.as_ref().map_or(false, |id| id == &id_score.id))?;
                let creator: User = serde_json::from_value(coin_raw.creator.clone()).ok()?;

                Some(CoinWithScore {
                    coin: OrderTokenResponse {
                        id: coin_raw.id.clone().unwrap_or_default(),
                        name: coin_raw.name.clone().unwrap_or_default(),
                        symbol: coin_raw.symbol.clone().unwrap_or_default(),
                        image_uri: coin_raw.image_uri.clone().unwrap_or_default(),
                        description: coin_raw.description.clone().unwrap_or_default(),
                        reply_count: coin_raw.reply_count.clone().unwrap_or_default(),
                        price: coin_raw.price.clone().unwrap_or_default(),
                        creator,
                        created_at: coin_raw.created_at.clone().unwrap_or_default(),
                    },
                    score: id_score.score.unwrap_or_default(),
                })
            })
            .collect();

        Ok(coins_with_score)
    }

    pub async fn get_creation_time_order_token(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_tokens(OrderType::CreationTime).await
    }

    pub async fn get_market_cap_order_token(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_tokens(OrderType::MarketCap).await
    }

    pub async fn get_reply_count_order_token(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_tokens(OrderType::ReplyCount).await
    }

    pub async fn get_latest_reply_order_token(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_tokens(OrderType::LatestReply).await
    }

    pub async fn get_bump_order_token(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_tokens(OrderType::Bump).await
    }

    // TODO: 어떤 식으로 정렬할지
    pub async fn search_order_tokens(&self, query: &str) -> Result<Vec<OrderTokenResponse>> {
        let coin_responses: Vec<OrderTokenResponseRaw> = sqlx::query_as(
            r#"
            SELECT 
                c.id,
                c.name,
                c.symbol,
                c.image_uri,
                c.description,
                c.created_at,
                COALESCE(crc.reply_count::TEXT, '0') as reply_count,
                COALESCE(cu.price::TEXT, '0') as price,
                json_build_object(
                    'nickname', a.nickname,
                    'image_uri', a.image_uri
                ) as creator
            FROM coin c
            LEFT JOIN account a ON c.creator = a.id
            LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
            LEFT JOIN curve cu ON c.id = cu.coin_id
            WHERE LOWER(c.name) LIKE $1 OR LOWER(c.symbol) LIKE $1
            ORDER BY cu.price DESC NULLS LAST
            LIMIT 50
            "#,
        )
        .bind(query)
        .fetch_all(&self.db.pool)
        .await
        .context("Failed to fetch coin information")?;
        info!("coin_responses: {:?}", coin_responses);
        let mut order_tokens: Vec<OrderTokenResponse> = coin_responses
            .into_iter()
            .filter_map(|raw| {
                let creator: User = serde_json::from_value(raw.creator).ok()?;

                Some(OrderTokenResponse {
                    id: raw.id.unwrap_or_default(),
                    name: raw.name.unwrap_or_default(),
                    symbol: raw.symbol.unwrap_or_default(),
                    image_uri: raw.image_uri.unwrap_or_default(),
                    description: raw.description.unwrap_or_default(),
                    reply_count: raw.reply_count.unwrap_or_default(),
                    price: raw.price.unwrap_or_default(),
                    creator,
                    created_at: raw.created_at.unwrap_or_default(),
                })
            })
            .collect();

        //sort 를 뭘로 할지 정해져야함
        order_tokens.sort_by(|a, b| {
            let price_a: f64 = a.price.parse().unwrap_or(0.0);
            let price_b: f64 = b.price.parse().unwrap_or(0.0);
            price_b
                .partial_cmp(&price_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(order_tokens)
    }
}

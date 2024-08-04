use anyhow::{Context, Result};
use sqlx::FromRow;
use std::sync::Arc;
use tracing::info;

use crate::{
    db::postgres::PostgresDatabase,
    types::{
        event::order::{OrderCoinResponse, OrderType, User},
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
    pub coin: OrderCoinResponse,
    pub score: String,
}
#[derive(Debug, Clone, sqlx::FromRow)]
struct OrderCoinResponseRaw {
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
    pub async fn get_order_coin_response(&self, coin_id: &str) -> Result<OrderCoinResponse> {
        let coin_response: OrderCoinResponseRaw = sqlx::query_as(
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

        Ok(OrderCoinResponse {
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
    async fn get_ordered_coins(&self, order_type: OrderType) -> Result<Vec<CoinWithScore>> {
        let id_scores_query  = match order_type {
            OrderType::CreationTime => 
                "SELECT id, created_at::TEXT as score FROM coin ORDER BY created_at DESC LIMIT $1"
            ,
            OrderType::MarketCap => 
                "SELECT coin_id as id, price::TEXT as score FROM curve ORDER BY price DESC LIMIT $1"   
            ,
            OrderType::ReplyCount => 
                "SELECT coin_id as id, reply_count::TEXT as score FROM coin_reply_count ORDER BY reply_count DESC LIMIT $1"
            ,
            OrderType::LatestReply => 
                "SELECT DISTINCT ON (coin_id) coin_id as id, created_at::TEXT as score FROM thread ORDER BY coin_id, created_at DESC LIMIT $1"
            ,
            OrderType::Bump => 
                "SELECT DISTINCT ON (coin_id) coin_id as id, created_at::TEXT as score FROM swap ORDER BY coin_id, created_at DESC LIMIT $1"
            ,
        };
        let coin_responses_query =  "SELECT c.id, c.name, c.symbol, c.image_uri, c.description,c.created_at, COALESCE(crc.reply_count::TEXT, '0') as reply_count, COALESCE(cu.price::TEXT, '0') as price, json_build_object('nickname', a.nickname, 'image_uri', a.image_uri) as creator FROM coin c LEFT JOIN account a ON c.creator = a.id LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id LEFT JOIN curve cu ON c.id = cu.coin_id WHERE c.id = ANY($1)";
        let id_scores: Vec<IdScore> = sqlx::query_as(id_scores_query)
            .bind(ORDER_LIMIT)
            .fetch_all(&self.db.pool)
            .await
            .context("Failed to fetch sorted coin IDs and scores")?;

        let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

        let coin_responses: Vec<OrderCoinResponseRaw> = sqlx::query_as(coin_responses_query)
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
                    coin: OrderCoinResponse {
                        id: coin_raw.id.clone().unwrap_or_default(),
                        name: coin_raw.name.clone().unwrap_or_default(),
                        symbol: coin_raw.symbol.clone().unwrap_or_default(),
                        image_uri: coin_raw.image_uri.clone().unwrap_or_default(),
                        description: coin_raw.description.clone().unwrap_or_default(),
                        reply_count: coin_raw.reply_count.clone().unwrap_or_default(),
                        price: coin_raw.price.clone().unwrap_or_default(),
                        creator,
                        created_at: coin_raw.created_at.clone().unwrap_or_default()
                    },
                    score: id_score.score.unwrap_or_default(),
                })
            })
            .collect();

        Ok(coins_with_score)
    }

    pub async fn get_creation_time_order_coin(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_coins(OrderType::CreationTime).await
    }

    pub async fn get_market_cap_order_coin(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_coins(OrderType::MarketCap).await
    }

    pub async fn get_reply_count_order_coin(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_coins(OrderType::ReplyCount).await
    }

    pub async fn get_latest_reply_order_coin(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_coins(OrderType::LatestReply).await
    }

    pub async fn get_bump_order_coin(&self) -> Result<Vec<CoinWithScore>> {
        self.get_ordered_coins(OrderType::Bump).await
    }
}

// Implement IdScore for the query result types
// impl OrderController {
//     pub fn new(db: Arc<PostgresDatabase>) -> Self {
//         OrderController { db }
//     }
//     pub async fn get_creation_time_order_coin(&self) -> Result<Vec<CoinWithScore>> {
//         // Step 1: Get sorted coin IDs and scores
//         let id_scores = sqlx::query!(
//             r#"
//             SELECT id, created_at::TEXT as score
//             FROM coin
//             ORDER BY created_at DESC
//             LIMIT $1
//             "#,
//             ORDER_LIMIT
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch sorted coin IDs and scores for creation time order")?;

//         let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

//         // Step 2: Get detailed coin information
//         // Step 2: Get detailed coin information
//         let coin_responses = sqlx::query_as!(
//             OrderCoinResponseRaw,
//             r#"
//         SELECT
//             c.id,
//             c.name,
//             c.symbol,
//             c.image_uri,
//             c.description,
//             COALESCE(crc.reply_count::TEXT, '0') as reply_count,
//             COALESCE(cu.price::TEXT, '0') as price,
//             json_build_object(
//                 'nickname', a.nickname,
//                 'image_uri', a.image_uri
//             ) as creator
//         FROM coin c
//         LEFT JOIN account a ON c.creator = a.id
//         LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
//         LEFT JOIN curve cu ON c.id = cu.coin_id
//         WHERE c.id = ANY($1)
//         "#,
//             &ids[..]
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch detailed coin information")?;

//         // Combine the results
//         let coins_with_score: Vec<CoinWithScore> = id_scores
//             .into_iter()
//             .filter_map(|id_score| {
//                 let coin_raw = coin_responses
//                     .iter()
//                     .find(|c| c.name.as_ref().map_or(false, |name| name == &id_score.id))?;
//                 let creator: User = serde_json::from_value(coin_raw.creator.clone()).ok()?;

//                 Some(CoinWithScore {
//                     coin: OrderCoinResponse {
//                         id: coin_raw.id.clone().unwrap_or_default(),
//                         name: coin_raw.name.clone().unwrap_or_default(),
//                         symbol: coin_raw.symbol.clone().unwrap_or_default(),
//                         image_uri: coin_raw.image_uri.clone().unwrap_or_default(),
//                         description: coin_raw.description.clone().unwrap_or_default(),
//                         reply_count: coin_raw.reply_count.clone().unwrap_or_default(),
//                         price: coin_raw.price.clone().unwrap_or_default(),
//                         creator,
//                     },
//                     score: id_score.score.unwrap_or_default(),
//                 })
//             })
//             .collect();

//         Ok(coins_with_score)
//     }
//     // pub async fn get_creation_time_order_coin(&self) -> Result<Vec<CoinWithScore>> {
//     //     let id_scores = sqlx::query!(
//     //         r#"
//     //         SELECT id, created_at::TEXT as score
//     //         FROM coin
//     //         ORDER BY created_at DESC
//     //         LIMIT $1
//     //         "#,
//     //         ORDER_LIMIT
//     //     )
//     //     .fetch_all(&self.db.pool)
//     //     .await
//     //     .context("Failed to fetch sorted coin IDs and scores for creation time order")?;

//     //     let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

//     //     let coins = sqlx::query_as!(
//     //         Coin,
//     //         r#"
//     //         SELECT *
//     //         FROM coin
//     //         WHERE id = ANY($1)
//     //         "#,
//     //         &ids[..]
//     //     )
//     //     .fetch_all(&self.db.pool)
//     //     .await
//     //     .context("Failed to fetch full coin information")?;

//     //     let coins_with_score = id_scores
//     //         .into_iter()
//     //         .map(|id_score| {
//     //             let coin = coins.iter().find(|c| c.id == id_score.id).unwrap().clone();
//     //             let score = id_score.score.unwrap_or_else(|| "0".to_string());

//     //             CoinWithScore { coin, score }
//     //         })
//     //         .collect();

//     //     Ok(coins_with_score)
//     // }

//     pub async fn get_market_cap_order_coin(&self) -> Result<Vec<CoinWithScore>> {
//         let id_scores = sqlx::query!(
//             r#"
//             SELECT coin_id as id, price::TEXT as score
//             FROM curve
//             ORDER BY price DESC
//             LIMIT $1
//             "#,
//             ORDER_LIMIT
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch sorted coin IDs and scores for market cap order")?;

//         let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

//         let coin_responses = sqlx::query_as!(
//             OrderCoinResponseRaw,
//             r#"
//         SELECT
//             c.id,
//             c.name,
//             c.symbol,
//             c.image_uri,
//             c.description,
//             COALESCE(crc.reply_count::TEXT, '0') as reply_count,
//             COALESCE(cu.price::TEXT, '0') as price,
//             json_build_object(
//                 'nickname', a.nickname,
//                 'image_uri', a.image_uri
//             ) as creator
//         FROM coin c
//         LEFT JOIN account a ON c.creator = a.id
//         LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
//         LEFT JOIN curve cu ON c.id = cu.coin_id
//         WHERE c.id = ANY($1)
//         "#,
//             &ids[..]
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch detailed coin information")?;

//         // Combine the results
//         let coins_with_score: Vec<CoinWithScore> = id_scores
//             .into_iter()
//             .filter_map(|id_score| {
//                 let coin_raw = coin_responses
//                     .iter()
//                     .find(|c| c.name.as_ref().map_or(false, |name| name == &id_score.id))?;
//                 let creator: User = serde_json::from_value(coin_raw.creator.clone()).ok()?;

//                 Some(CoinWithScore {
//                     coin: OrderCoinResponse {
//                         id: coin_raw.id.clone().unwrap_or_default(),
//                         name: coin_raw.name.clone().unwrap_or_default(),
//                         symbol: coin_raw.symbol.clone().unwrap_or_default(),
//                         image_uri: coin_raw.image_uri.clone().unwrap_or_default(),
//                         description: coin_raw.description.clone().unwrap_or_default(),
//                         reply_count: coin_raw.reply_count.clone().unwrap_or_default(),
//                         price: coin_raw.price.clone().unwrap_or_default(),
//                         creator,
//                     },
//                     score: id_score.score.unwrap_or_default(),
//                 })
//             })
//             .collect();
//         Ok(coins_with_score)
//     }

//     pub async fn get_reply_count_order_coin(&self) -> Result<Vec<CoinWithScore>> {
//         let id_scores = sqlx::query!(
//             r#"
//             SELECT coin_id as id, reply_count::TEXT as score
//             FROM coin_reply_count
//             ORDER BY reply_count DESC
//             LIMIT $1
//             "#,
//             ORDER_LIMIT
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch sorted coin IDs and scores for reply count order")?;

//         let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

//         let coin_responses = sqlx::query_as!(
//             OrderCoinResponseRaw,
//             r#"
//         SELECT
//             c.id,
//             c.name,
//             c.symbol,
//             c.image_uri,
//             c.description,
//             COALESCE(crc.reply_count::TEXT, '0') as reply_count,
//             COALESCE(cu.price::TEXT, '0') as price,
//             json_build_object(
//                 'nickname', a.nickname,
//                 'image_uri', a.image_uri
//             ) as creator
//         FROM coin c
//         LEFT JOIN account a ON c.creator = a.id
//         LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
//         LEFT JOIN curve cu ON c.id = cu.coin_id
//         WHERE c.id = ANY($1)
//         "#,
//             &ids[..]
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch detailed coin information")?;

//         // Combine the results
//         let coins_with_score: Vec<CoinWithScore> = id_scores
//             .into_iter()
//             .filter_map(|id_score| {
//                 let coin_raw = coin_responses
//                     .iter()
//                     .find(|c| c.name.as_ref().map_or(false, |name| name == &id_score.id))?;
//                 let creator: User = serde_json::from_value(coin_raw.creator.clone()).ok()?;

//                 Some(CoinWithScore {
//                     coin: OrderCoinResponse {
//                         id: coin_raw.id.clone().unwrap_or_default(),
//                         name: coin_raw.name.clone().unwrap_or_default(),
//                         symbol: coin_raw.symbol.clone().unwrap_or_default(),
//                         image_uri: coin_raw.image_uri.clone().unwrap_or_default(),
//                         description: coin_raw.description.clone().unwrap_or_default(),
//                         reply_count: coin_raw.reply_count.clone().unwrap_or_default(),
//                         price: coin_raw.price.clone().unwrap_or_default(),
//                         creator,
//                     },
//                     score: id_score.score.unwrap_or_default(),
//                 })
//             })
//             .collect();
//         Ok(coins_with_score)
//     }

//     pub async fn get_latest_reply_order_coin(&self) -> Result<Vec<CoinWithScore>> {
//         let id_scores = sqlx::query!(
//             r#"
//             SELECT DISTINCT ON (coin_id)
//                 coin_id as id,
//                 created_at::TEXT as score
//             FROM thread
//             ORDER BY coin_id, created_at DESC
//             LIMIT $1
//             "#,
//             ORDER_LIMIT
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch sorted coin IDs and scores for latest reply order")?;

//         let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

//         let coin_responses = sqlx::query_as!(
//             OrderCoinResponseRaw,
//             r#"
//         SELECT
//             c.id,
//             c.name,
//             c.symbol,
//             c.image_uri,
//             c.description,
//             COALESCE(crc.reply_count::TEXT, '0') as reply_count,
//             COALESCE(cu.price::TEXT, '0') as price,
//             json_build_object(
//                 'nickname', a.nickname,
//                 'image_uri', a.image_uri
//             ) as creator
//         FROM coin c
//         LEFT JOIN account a ON c.creator = a.id
//         LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
//         LEFT JOIN curve cu ON c.id = cu.coin_id
//         WHERE c.id = ANY($1)
//         "#,
//             &ids[..]
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch detailed coin information")?;

//         // Combine the results
//         let coins_with_score: Vec<CoinWithScore> = id_scores
//             .into_iter()
//             .filter_map(|id_score| {
//                 let coin_raw = coin_responses
//                     .iter()
//                     .find(|c| c.name.as_ref().map_or(false, |name| name == &id_score.id))?;
//                 let creator: User = serde_json::from_value(coin_raw.creator.clone()).ok()?;

//                 Some(CoinWithScore {
//                     coin: OrderCoinResponse {
//                         id: coin_raw.id.clone().unwrap_or_default(),
//                         name: coin_raw.name.clone().unwrap_or_default(),
//                         symbol: coin_raw.symbol.clone().unwrap_or_default(),
//                         image_uri: coin_raw.image_uri.clone().unwrap_or_default(),
//                         description: coin_raw.description.clone().unwrap_or_default(),
//                         reply_count: coin_raw.reply_count.clone().unwrap_or_default(),
//                         price: coin_raw.price.clone().unwrap_or_default(),
//                         creator,
//                     },
//                     score: id_score.score.unwrap_or_default(),
//                 })
//             })
//             .collect();
//         Ok(coins_with_score)
//     }

//     pub async fn get_bump_order_coin(&self) -> Result<Vec<CoinWithScore>> {
//         let id_scores = sqlx::query!(
//             r#"
//             SELECT DISTINCT ON (coin_id)
//                 coin_id as id,
//                 created_at::TEXT as score
//             FROM swap
//             ORDER BY coin_id, created_at DESC
//             LIMIT $1
//             "#,
//             ORDER_LIMIT
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch sorted coin IDs and scores for bump order")?;

//         let ids: Vec<String> = id_scores.iter().map(|row| row.id.clone()).collect();

//         let coin_responses = sqlx::query_as!(
//             OrderCoinResponseRaw,
//             r#"
//         SELECT
//             c.id,
//             c.name,
//             c.symbol,
//             c.image_uri,
//             c.description,
//             COALESCE(crc.reply_count::TEXT, '0') as reply_count,
//             COALESCE(cu.price::TEXT, '0') as price,
//             json_build_object(
//                 'nickname', a.nickname,
//                 'image_uri', a.image_uri
//             ) as creator
//         FROM coin c
//         LEFT JOIN account a ON c.creator = a.id
//         LEFT JOIN coin_reply_count crc ON c.id = crc.coin_id
//         LEFT JOIN curve cu ON c.id = cu.coin_id
//         WHERE c.id = ANY($1)
//         "#,
//             &ids[..]
//         )
//         .fetch_all(&self.db.pool)
//         .await
//         .context("Failed to fetch detailed coin information")?;

//         // Combine the results
//         let coins_with_score: Vec<CoinWithScore> = id_scores
//             .into_iter()
//             .filter_map(|id_score| {
//                 let coin_raw = coin_responses
//                     .iter()
//                     .find(|c| c.name.as_ref().map_or(false, |name| name == &id_score.id))?;
//                 let creator: User = serde_json::from_value(coin_raw.creator.clone()).ok()?;

//                 Some(CoinWithScore {
//                     coin: OrderCoinResponse {
//                         id: coin_raw.id.clone().unwrap_or_default(),
//                         name: coin_raw.name.clone().unwrap_or_default(),
//                         symbol: coin_raw.symbol.clone().unwrap_or_default(),
//                         image_uri: coin_raw.image_uri.clone().unwrap_or_default(),
//                         description: coin_raw.description.clone().unwrap_or_default(),
//                         reply_count: coin_raw.reply_count.clone().unwrap_or_default(),
//                         price: coin_raw.price.clone().unwrap_or_default(),
//                         creator,
//                     },
//                     score: id_score.score.unwrap_or_default(),
//                 })
//             })
//             .collect();

//         Ok(coins_with_score)
//     }
// }

// pub async fn get_market_cap_order_coin_join(&self) -> Result<Vec<Coin>> {
//     let coins = sqlx::query_as!(
//         Coin,
//         r#"
//         SELECT c.*
//         FROM coin c
//         JOIN (
//             SELECT coin_id, price
//             FROM curve
//             ORDER BY price DESC
//             LIMIT 50
//         ) t ON c.id = t.coin_id
//         ORDER BY t.price DESC
//         "#
//     )
//     .fetch_all(&self.db.pool)
//     .await
//     .context("Failed to fetch coins ordered by market cap")?;
//     Ok(coins)
// }

// pub async fn get_bump_order_coin_join(&self) -> Result<Vec<Coin>> {
//     let coins = sqlx::query_as!(
//         Coin,
//         r#"
//         SELECT c.*
//         FROM coin c
//         JOIN (
//             SELECT DISTINCT ON (coin_id) coin_id, id
//             FROM swap
//             ORDER BY coin_id, id DESC
//         ) s ON c.id = s.coin_id
//         JOIN curve cv ON c.id = cv.coin_id
//         ORDER BY s.id DESC, cv.price DESC
//         LIMIT 50
//         "#
//     )
//     .fetch_all(&self.db.pool)
//     .await
//     .context("Failed to fetch coins ordered by bump")?;
//     Ok(coins)
// }

// pub async fn get_reply_count_order_coin_join(&self) -> Result<Vec<Coin>> {
//     let coins = sqlx::query_as!(
//         Coin,
//         r#"
//         SELECT c.*
//         FROM coin c
//         JOIN coin_reply_count crc ON c.id = crc.coin_id
//         ORDER BY crc.reply_count DESC
//         LIMIT 50
//         "#
//     )
//     .fetch_all(&self.db.pool)
//     .await
//     .context("Failed to fetch coins ordered by reply count")?;
//     Ok(coins)
// }

// pub async fn get_latest_reply_order_coin_join(&self) -> Result<Vec<Coin>> {
//     let coins = sqlx::query_as!(
//         Coin,
//         r#"
//         SELECT c.*
//         FROM coin c
//         JOIN (
//             SELECT coin_id, MAX(created_at) as latest_thread
//             FROM thread
//             GROUP BY coin_id
//             ORDER BY latest_thread DESC
//             LIMIT 50
//         ) t ON c.id = t.coin_id
//         ORDER BY t.latest_thread DESC
//         "#
//     )
//     .fetch_all(&self.db.pool)
//     .await
//     .context("Failed to fetch coins ordered by latest reply")?;
//     Ok(coins)
// }

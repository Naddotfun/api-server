use anyhow::Result;
use bigdecimal::{BigDecimal, Zero};
use std::sync::Arc;

use crate::{
    db::postgres::PostgresDatabase,
    types::{
        model::{Account, Coin, Curve, Thread},
        HoldCoinResponse,
    },
};

pub struct ProfileController {
    pub db: Arc<PostgresDatabase>,
}

impl ProfileController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        ProfileController { db }
    }

    pub async fn get_profile(&self, identifier: &str) -> Result<Account> {
        let account = sqlx::query_as!(
            Account,
            r#"
            SELECT id, nickname, bio, image_uri, follower_count, following_count, like_count
            FROM account
            WHERE nickname = $1 OR id = $1
            "#,
            identifier
        )
        .fetch_one(&self.db.pool)
        .await?;

        Ok(account)
    }

    pub async fn get_account_holding_coin(&self, address: &str) -> Result<Vec<HoldCoinResponse>> {
        let holdings = sqlx::query!(
            r#"
            SELECT 
               c.*,
                b.amount AS balance,
                COALESCE(cu.price, 0) AS price
            FROM 
                balance b
            JOIN 
                coin c ON b.coin_id = c.id
            LEFT JOIN 
                curve cu ON c.id = cu.coin_id
            WHERE 
                b.account = $1
            "#,
            address
        )
        .fetch_all(&self.db.pool)
        .await?;

        let mut results: Vec<HoldCoinResponse> = holdings
            .into_iter()
            .map(|row| {
                let coin = Coin {
                    id: row.id,
                    name: row.name,
                    symbol: row.symbol,
                    image_uri: row.image_uri,
                    description: row.description,
                    creator: row.creator,
                    created_at: row.created_at,
                    twitter: row.twitter,
                    telegram: row.telegram,
                    website: row.website,
                    is_listing: row.is_listing,
                    create_transaction_hash: row.create_transaction_hash,
                    is_updated: row.is_updated,
                };

                let balance = row.balance;
                let price = row.price.unwrap_or_else(BigDecimal::zero);

                HoldCoinResponse {
                    coin,
                    balance,
                    price,
                }
            })
            .collect();
        results.sort_by(|a, b| {
            let value_a = &a.price * &a.balance;
            let value_b = &b.price * &b.balance;
            value_b
                .partial_cmp(&value_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }

    pub async fn get_account_replies(&self, address: &str) -> Result<Vec<Thread>> {
        let replies = sqlx::query_as!(
            Thread,
            r#"
            SELECT *
            FROM thread
            WHERE author_id = $1
            ORDER BY created_at DESC
            "#,
            address
        )
        .fetch_all(&self.db.pool)
        .await?;

        Ok(replies)
    }

    pub async fn get_account_created_coins(&self, address: &str) -> Result<Vec<Coin>> {
        let coins = sqlx::query_as!(
            Coin,
            r#"
            SELECT *
            FROM coin
            WHERE creator = $1
            ORDER BY created_at DESC
            "#,
            address
        )
        .fetch_all(&self.db.pool)
        .await?;

        Ok(coins)
    }

    pub async fn get_account_followers(&self, address: &str) -> Result<Vec<Account>> {
        let followers = sqlx::query_as!(
            Account,
            r#"
            SELECT a.id, a.nickname, a.bio, a.image_uri, a.follower_count, a.following_count, a.like_count
            FROM follow f
            JOIN account a ON f.following_id = a.id
            WHERE f.follower_id = $1
            "#,
            address
        )
        .fetch_all(&self.db.pool)
        .await?;

        Ok(followers)
    }

    pub async fn get_account_follwing(&self, address: &str) -> Result<Vec<Account>> {
        let followings = sqlx::query_as!(
            Account,
            r#"
            SELECT a.id, a.nickname, a.bio, a.image_uri, a.follower_count, a.following_count, a.like_count
            FROM follow f
            JOIN account a ON f.follower_id = a.id
            WHERE f.following_id = $1
            "#,
            address
        )
        .fetch_all(&self.db.pool)
        .await?;

        Ok(followings)
    }
}

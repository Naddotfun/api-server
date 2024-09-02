use anyhow::Result;
use bigdecimal::{BigDecimal, Zero};
use sqlx::FromRow;
use std::sync::Arc;

use crate::{
    db::postgres::PostgresDatabase,
    types::{
        model::{Account, Curve, Thread, Token},
        profile::{HoldToken, Identifier},
    },
};

pub struct ProfileController {
    pub db: Arc<PostgresDatabase>,
}
#[derive(FromRow)]
struct TokenHoldingRow {
    // Token fields
    id: String,
    name: String,
    symbol: String,
    image_uri: String,
    description: Option<String>,
    creator: String,
    created_at: i64,
    twitter: Option<String>,
    telegram: Option<String>,
    website: Option<String>,
    is_listing: bool,
    create_transaction_hash: String,
    is_updated: bool,
    // Additional fields
    balance: BigDecimal,
    price: Option<BigDecimal>,
    pair: Option<String>,
}

#[derive(FromRow)]
struct TokenHoldingRecord {
    token: Token,
    balance: BigDecimal,
    price: BigDecimal,
}

impl ProfileController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        ProfileController { db }
    }
    pub async fn get_profile(&self, identifier: &Identifier) -> Result<Account> {
        let account = match identifier {
            Identifier::Nickname(nickname) => {
                sqlx::query_as!(
                    Account,
                    "SELECT * FROM account WHERE nickname = $1",
                    nickname
                )
                .fetch_one(&self.db.pool)
                .await?
            }
            Identifier::Address(address) => {
                sqlx::query_as!(Account, "SELECT * FROM account WHERE id = $1", address)
                    .fetch_one(&self.db.pool)
                    .await?
            }
        };

        Ok(account)
    }

    pub async fn get_holding_token(&self, identifier: &Identifier) -> Result<Vec<HoldToken>> {
        let holdings = match identifier {
            Identifier::Nickname(nickname) => {
                sqlx::query_as!(
                    TokenHoldingRow,
                    r#"
                    SELECT 
                       t.*,
                        b.amount AS balance,
                        COALESCE(cu.price, 0) AS price
                    FROM 
                        balance b
                    JOIN 
                        account a ON b.account_id = a.id
                    JOIN 
                        token t ON b.token_id = t.id
                    LEFT JOIN 
                        curve cu ON t.id = cu.token_id
                    WHERE 
                        a.nickname = $1
                    "#,
                    nickname
                )
                .fetch_all(&self.db.pool)
                .await?
            }
            Identifier::Address(address) => {
                sqlx::query_as!(
                    TokenHoldingRow,
                    r#"
                    SELECT 
                       t.*,
                        b.amount AS balance,
                        COALESCE(cu.price, 0) AS price
                    FROM 
                        balance b
                    JOIN 
                        token t ON b.token_id = t.id
                    LEFT JOIN 
                        curve cu ON t.id = cu.token_id
                    WHERE 
                        b.account_id = $1
                    "#,
                    address
                )
                .fetch_all(&self.db.pool)
                .await?
            }
        };

        let mut results: Vec<TokenHoldingRecord> = holdings
            .into_iter()
            .map(|row| {
                let token = Token {
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
                    pair: row.pair,
                };

                TokenHoldingRecord {
                    token,
                    balance: row.balance,
                    price: row.price.unwrap_or(BigDecimal::zero()),
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

        let results: Vec<HoldToken> = results
            .into_iter()
            .map(|record| HoldToken {
                token: record.token,
                balance: record.balance.to_string(),
                price: record.price.to_string(),
            })
            .collect();

        Ok(results)
    }

    pub async fn get_replies(&self, identifier: &Identifier) -> Result<Vec<Thread>> {
        let replies = match identifier {
            Identifier::Nickname(nickname) => {
                sqlx::query_as!(
                    Thread,
                    r#"
                    SELECT t.*
                    FROM thread t
                    JOIN account a ON t.author_id = a.id
                    WHERE a.nickname = $1
                    ORDER BY t.created_at DESC
                    "#,
                    nickname
                )
                .fetch_all(&self.db.pool)
                .await?
            }
            Identifier::Address(address) => {
                sqlx::query_as!(
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
                .await?
            }
        };

        Ok(replies)
    }

    pub async fn get_created_tokens(&self, identifier: &Identifier) -> Result<Vec<Token>> {
        let tokens = match identifier {
            Identifier::Nickname(nickname) => {
                sqlx::query_as!(
                    Token,
                    r#"
                    SELECT t.*
                    FROM token t
                    JOIN account a ON t.creator = a.id
                    WHERE a.nickname = $1
                    ORDER BY t.created_at DESC
                    "#,
                    nickname
                )
                .fetch_all(&self.db.pool)
                .await?
            }
            Identifier::Address(address) => {
                sqlx::query_as!(
                    Token,
                    r#"
                    SELECT *
                    FROM token
                    WHERE creator = $1
                    ORDER BY created_at DESC
                    "#,
                    address
                )
                .fetch_all(&self.db.pool)
                .await?
            }
        };

        Ok(tokens)
    }

    pub async fn get_followers(&self, identifier: &Identifier) -> Result<Vec<Account>> {
        let followers = match identifier {
            Identifier::Nickname(nickname) => {
                sqlx::query_as!(
                    Account,
                    r#"
                    SELECT a2.id, a2.nickname, a2.bio, a2.image_uri, a2.follower_count, a2.following_count, a2.like_count
                    FROM follow f
                    JOIN account a1 ON f.following_id = a1.id
                    JOIN account a2 ON f.follower_id = a2.id
                    WHERE a1.nickname = $1
                    "#,
                    nickname
                )
                .fetch_all(&self.db.pool)
                .await?
            },
            Identifier::Address(address) => {
                sqlx::query_as!(
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
                .await?
            }
        };

        Ok(followers)
    }

    pub async fn get_following(&self, identifier: &Identifier) -> Result<Vec<Account>> {
        let followings = match identifier {
            Identifier::Nickname(nickname) => {
                sqlx::query_as!(
                    Account,
                    r#"
                    SELECT a2.id, a2.nickname, a2.bio, a2.image_uri, a2.follower_count, a2.following_count, a2.like_count
                    FROM follow f
                    JOIN account a1 ON f.follower_id = a1.id
                    JOIN account a2 ON f.following_id = a2.id
                    WHERE a1.nickname = $1
                    "#,
                    nickname
                )
                .fetch_all(&self.db.pool)
                .await?
            },
            Identifier::Address(address) => {
                sqlx::query_as!(
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
                .await?
            }
        };

        Ok(followings)
    }
}

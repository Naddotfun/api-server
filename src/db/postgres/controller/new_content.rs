use anyhow::Result;
use tracing::info;

use std::sync::Arc;

use crate::{
    db::postgres::PostgresDatabase,
    types::event::{NewSwapMessage, NewTokenMessage, TokenInfo, UserInfo},
};

pub struct InitContentController {
    pub db: Arc<PostgresDatabase>,
}

impl InitContentController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        InitContentController { db }
    }

    pub async fn get_latest_buy(&self) -> Result<Option<NewSwapMessage>> {
        let result = sqlx::query!(
            r#"
            SELECT
                s.is_buy,
                s.nad_amount,
                a.nickname as user_nickname,
                a.image_uri as user_image_uri,
                t.symbol as token_symbol,
                t.image_uri as token_image_uri,
                t.id as token_id
            FROM swap s
            JOIN account a ON s.sender = a.id
            JOIN token t ON s.token_id = t.id
            WHERE s.is_buy = true
            ORDER BY s.created_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db.pool)
        .await?;
        
        Ok(result.map(|row| NewSwapMessage {
            user_info: UserInfo {
                nickname: row.user_nickname,
                image_uri: row.user_image_uri,
            },
            token_info: TokenInfo {
                id: row.token_id,
                symbol: row.token_symbol,
                image_uri: row.token_image_uri,
            },
            is_buy: row.is_buy,
            nad_amount: row.nad_amount.to_string(),
        }))
    }

    pub async fn get_latest_sell(&self) -> Result<Option<NewSwapMessage>> {
        let result = sqlx::query!(
            r#"
            SELECT
                s.is_buy,
                s.nad_amount,
                a.nickname as user_nickname,
                a.image_uri as user_image_uri,
                t.symbol as token_symbol,
                t.image_uri as token_image_uri,
                t.id as token_id
            FROM swap s
            JOIN account a ON s.sender = a.id
            JOIN token t ON s.token_id = t.id
            WHERE s.is_buy = false
            ORDER BY s.created_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(result.map(|row| NewSwapMessage {
            user_info: UserInfo {
                nickname: row.user_nickname,
                image_uri: row.user_image_uri,
            },
            token_info: TokenInfo {
                id: row.token_id,
                symbol: row.token_symbol,
                image_uri: row.token_image_uri,
            },
            is_buy: row.is_buy,
            nad_amount: row.nad_amount.to_string(),
        }))
    }

    pub async fn get_latest_new_token(&self) -> Result<Option<NewTokenMessage>> {
        let result = sqlx::query!(
            r#"
            SELECT 
                t.symbol,
                t.image_uri as token_image_uri,
                t.created_at,
                t.id as token_id,
                a.nickname as user_nickname,
                a.image_uri as user_image_uri
            FROM token t
            JOIN account a ON t.creator = a.id
            ORDER BY t.created_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(result.map(|row| NewTokenMessage {
            user_info: UserInfo {
                nickname: row.user_nickname,
                image_uri: row.user_image_uri,
            },
            id: row.token_id,
            symbol: row.symbol,
            image_uri: row.token_image_uri,
            created_at: row.created_at,
        }))
    }
}

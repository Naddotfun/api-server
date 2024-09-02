use anyhow::Result;

use std::sync::Arc;

use crate::{
    db::postgres::PostgresDatabase,
    types::event::{TokenAndUserInfo, TokenInfo, UserInfo},
};

pub struct InfoController {
    pub db: Arc<PostgresDatabase>,
}

impl InfoController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        InfoController { db }
    }
    pub async fn get_user(&self, account_id: &str) -> Result<UserInfo> {
        let creator = sqlx::query_as!(
            UserInfo,
            "SELECT nickname, image_uri
             FROM account WHERE id = $1",
            account_id
        )
        .fetch_one(&self.db.pool)
        .await?;
        Ok(creator)
    }
    pub async fn get_token_info(&self, token_id: &str) -> Result<TokenInfo> {
        let token_info = sqlx::query_as!(
            TokenInfo,
            "SELECT id,symbol, image_uri FROM token WHERE id = $1",
            token_id
        )
        .fetch_one(&self.db.pool)
        .await?;

        Ok(token_info)
    }

    pub async fn get_token_and_user_info(
        &self,
        token_id: &str,
        user_id: &str,
    ) -> Result<TokenAndUserInfo> {
        sqlx::query_as!(
            TokenAndUserInfo,
            r#"
            SELECT 
                t.id as token_id,
                t.symbol as token_symbol,
                t.image_uri as token_image_uri,
                a.nickname as user_nickname,
                a.image_uri as user_image_uri
            FROM token t
            JOIN account a ON a.id = $2
            WHERE t.id = $1
            "#,
            token_id,
            user_id
        )
        .fetch_one(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch token and user info: {}", e))
    }
}

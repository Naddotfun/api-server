use anyhow::Result;
use sqlx::FromRow;
use std::sync::Arc;

use crate::{
    db::postgres::PostgresDatabase,
    types::event::{order::CreateSwapCoinInfo, CoinAndUserInfo, User},
};

pub struct InfoController {
    pub db: Arc<PostgresDatabase>,
}

impl InfoController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        InfoController { db }
    }
    pub async fn get_user(&self, account_id: &str) -> Result<User> {
        let creator = sqlx::query_as!(
            User,
            "SELECT nickname, image_uri
             FROM account WHERE id = $1",
            account_id
        )
        .fetch_one(&self.db.pool)
        .await?;
        Ok(creator)
    }
    pub async fn get_coin_info(&self, coin_id: &str) -> Result<CreateSwapCoinInfo> {
        let coin_info = sqlx::query_as!(
            CreateSwapCoinInfo,
            "SELECT symbol, image_uri FROM coin WHERE id = $1",
            coin_id
        )
        .fetch_one(&self.db.pool)
        .await?;

        Ok(coin_info)
    }

    pub async fn get_coin_and_user_info(
        &self,
        coin_id: &str,
        user_id: &str,
    ) -> Result<CoinAndUserInfo> {
        sqlx::query_as!(
            CoinAndUserInfo,
            r#"
            SELECT 
                c.id as coin_id,
                c.symbol as coin_symbol,
                c.image_uri as coin_image_uri,
                a.nickname as user_nickname,
                a.image_uri as user_image_uri
            FROM coin c
            JOIN account a ON a.id = $2
            WHERE c.id = $1
            "#,
            coin_id,
            user_id
        )
        .fetch_one(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch coin and user info: {}", e))
    }
}

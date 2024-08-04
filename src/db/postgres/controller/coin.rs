use anyhow::Result;
use std::sync::Arc;

use crate::{db::postgres::PostgresDatabase, types::model::Coin};

pub struct CoinController {
    pub db: Arc<PostgresDatabase>,
}

impl CoinController {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        CoinController { db }
    }

    pub async fn get_coin_by_id(&self, coin_id: &str) -> Result<Coin> {
        let coin = sqlx::query_as!(Coin, "SELECT * FROM coin WHERE id = $1", coin_id)
            .fetch_one(&self.db.pool)
            .await?;
        Ok(coin)
    }
}

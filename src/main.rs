use std::sync::Arc;

use anyhow::Result;

use api_server::{
    db::{postgres::PostgresDatabase, redis::RedisDatabase},
    event::{
        new_content::{self, NewContentEventProducer},
        order::{self, OrderEventProducer},
        token::{self, TokenEventProducer},
    },
    server,
};
use tokio::task::JoinSet;
use tracing::{info, warn};
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let mut set = JoinSet::new();
    let postgres = Arc::new(PostgresDatabase::new().await);
    let redis = Arc::new(RedisDatabase::new().await);

    let coin_event_producer = Arc::new(TokenEventProducer::new(postgres.clone()));
    let order_event_porducer = Arc::new(OrderEventProducer::new(redis.clone(), postgres.clone()));
    let new_content_producer = Arc::new(NewContentEventProducer::new(
        postgres.clone(),
        redis.clone(),
    ));
    set.spawn(order::main(order_event_porducer.clone()));
    set.spawn(token::main(coin_event_producer.clone()));
    set.spawn(new_content::main(
        new_content_producer.clone(),
        redis.clone(),
    ));

    set.spawn(server::main(
        postgres.clone(),
        redis.clone(),
        order_event_porducer.clone(),
        coin_event_producer.clone(),
        new_content_producer.clone(),
    ));
    while let Some(res) = set.join_next().await {
        match res {
            Ok(_) => info!("Task completed successfully"),
            Err(e) => warn!("Task panicked: {:?}", e),
        }
    }
    warn!("Main end");
    Ok(())
}

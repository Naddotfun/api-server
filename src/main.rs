use std::{env, sync::Arc};

use anyhow::Result;

use read_engine::{
    db::{postgres::PostgresDatabase, redis::RedisDatabase},
    event::{
        coin::{self, CoinEventProducer},
        order::{self, OrderEventProducer},
    },
    server,
    types::event::order::OrderMessage,
};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::JoinSet,
};
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

    let coin_event_producer = Arc::new(CoinEventProducer::new(postgres.clone()));
    let order_event_porducer = Arc::new(OrderEventProducer::new(redis.clone(), postgres.clone()));

    set.spawn(order::main(order_event_porducer.clone()));
    set.spawn(coin::main(coin_event_producer.clone()));
    // let global_event_controller = Arc::new(GlobalEventController::new(100));
    // set.spawn(coin::main(coin_event_producer.clone(), postgres.clone()));

    set.spawn(server::main(
        postgres.clone(),
        redis.clone(),
        order_event_porducer.clone(),
        coin_event_producer.clone(),
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

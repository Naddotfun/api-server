use std::sync::Arc;

use crate::{
    db::{postgres::PostgresDatabase, redis::RedisDatabase},
    event::{
        coin::CoinEventProducer, new_content::NewContentEventProducer, order::OrderEventProducer,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub postgres: Arc<PostgresDatabase>,
    pub redis: Arc<RedisDatabase>,
    pub order_event_producer: Arc<OrderEventProducer>,
    pub coin_event_producer: Arc<CoinEventProducer>,
    pub new_content_producer: Arc<NewContentEventProducer>,
}

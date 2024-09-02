use std::sync::Arc;

use crate::{
    db::{postgres::PostgresDatabase, redis::RedisDatabase},
    event::{
        new_content::NewContentEventProducer, order::OrderEventProducer, token::TokenEventProducer,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub postgres: Arc<PostgresDatabase>,
    pub redis: Arc<RedisDatabase>,
    pub order_event_producer: Arc<OrderEventProducer>,
    pub token_event_producer: Arc<TokenEventProducer>,
    pub new_content_producer: Arc<NewContentEventProducer>,
}

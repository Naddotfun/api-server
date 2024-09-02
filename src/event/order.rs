use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    constant::change_channels::{CURVE, SWAP, THREAD, TOKEN, TOKEN_REPLIES_COUNT},
    db::{
        postgres::{
            controller::{info::InfoController, order::OrderController},
            PostgresDatabase,
        },
        redis::RedisDatabase,
    },
    types::{
        event::{
            capture::OrderEventCapture,
            order::{OrderMessage, OrderTokenResponse, OrderType},
            NewSwapMessage, NewTokenMessage, SendMessageType,
        },
        model::{
            Curve, CurveWrapper, FromValue, Swap, SwapWrapper, Token, TokenReplyCount,
            TokenReplyCountWrapper, TokenWrapper,
        },
    },
};
use anyhow::{Context, Result};
use chrono::Utc;
use futures::{future::try_join_all, StreamExt};

use serde_json::Value;
use sqlx::postgres::{PgListener, PgNotification};
use tokio::{
    sync::{
        broadcast::{self, Receiver, Sender},
        RwLock,
    },
    time::sleep,
};
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip(producer))]
pub async fn main(producer: Arc<OrderEventProducer>) -> Result<()> {
    info!("Starting event capture");
    producer
        .initialize()
        .await
        .context("Failed to initialize producer")?;

    loop {
        match producer.change_data_capture().await {
            Ok(_) => {
                warn!("Event capture completed unexpectedly");
                break;
            }
            Err(e) => {
                error!("Error in change_data_capture: {:?}", e);
                info!("Retrying in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    error!("Event capture ended");
    Ok(())
}

pub struct OrderReceiver {
    receiver: Receiver<OrderMessage>,
    order_type: OrderType,
    controller: Arc<OrderEventProducer>,
}

impl OrderReceiver {
    pub async fn recv(&mut self) -> Option<OrderMessage> {
        self.receiver.recv().await.ok()
    }
}
impl Drop for OrderReceiver {
    fn drop(&mut self) {
        let controller = self.controller.clone();
        let order_type = self.order_type.clone();
        tokio::spawn(async move {
            controller.decrement_receiver_count(order_type).await;
        });
    }
}
#[derive(Clone)]
pub struct OrderEventProducer {
    redis: Arc<RedisDatabase>,
    db: Arc<PostgresDatabase>,
    order_senders: Arc<RwLock<HashMap<OrderType, (Sender<OrderMessage>, usize)>>>,
    total_channels: Arc<AtomicUsize>,
}

impl OrderEventProducer {
    pub fn new(redis: Arc<RedisDatabase>, db: Arc<PostgresDatabase>) -> Self {
        Self {
            redis,
            db,
            order_senders: Arc::new(RwLock::new(HashMap::new())),
            total_channels: Arc::new(AtomicUsize::new(0)),
        }
    }
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Order");
        let order_controller = OrderController::new(self.db.clone());

        let tasks = vec![
            self.initial_ordering(&order_controller, OrderType::CreationTime),
            self.initial_ordering(&order_controller, OrderType::MarketCap),
            self.initial_ordering(&order_controller, OrderType::Bump),
            self.initial_ordering(&order_controller, OrderType::ReplyCount),
            self.initial_ordering(&order_controller, OrderType::LatestReply),
        ];

        try_join_all(tasks).await?;
        info!("Initialized Ordering");
        Ok(())
    }
    async fn initial_ordering(
        &self,
        controller: &OrderController,
        order_type: OrderType,
    ) -> Result<()> {
        let tokens_result = match order_type {
            OrderType::CreationTime => controller.get_creation_time_order_token().await,
            OrderType::MarketCap => controller.get_market_cap_order_token().await,
            OrderType::Bump => controller.get_bump_order_token().await,
            OrderType::ReplyCount => controller.get_reply_count_order_token().await,
            OrderType::LatestReply => controller.get_latest_reply_order_token().await,
        };

        // 에러 처리
        let tokens = tokens_result.map_err(|e| {
            error!("Failed to get {:?} order tokens: {:?}", order_type, e);
            anyhow::anyhow!("Failed to get {:?} order tokens", order_type)
        })?;

        // 빈 벡터 체크
        if tokens.is_empty() {
            info!("No tokens found for {:?} order", order_type);
            return Ok(());
        }

        let redis_result = match order_type {
            OrderType::CreationTime => self.redis.set_creation_time_order(tokens).await,
            OrderType::MarketCap => self.redis.set_market_cap_order(tokens).await,
            OrderType::Bump => self.redis.set_bump_order(tokens).await,
            OrderType::ReplyCount => self.redis.set_reply_count_order(tokens).await,
            OrderType::LatestReply => self.redis.set_last_reply_order(tokens).await,
        };

        // Redis 설정 에러 처리
        redis_result.map_err(|e| {
            error!("Failed to set {:?} order in Redis: {:?}", order_type, e);
            anyhow::anyhow!("Failed to set {:?} order in Redis", order_type)
        })?;

        info!("Successfully set {:?} order in Redis", order_type);
        Ok(())
    }
    #[instrument(skip(self))]
    pub async fn change_data_capture(&self) -> Result<()> {
        let mut listener = PgListener::connect_with(&self.db.pool).await?;
        listener
            .listen_all(vec![TOKEN, SWAP, CURVE, TOKEN_REPLIES_COUNT])
            .await?;
        let mut stream = listener.into_stream();
        info!("Order event capture Start");

        while let Some(notification) = stream.next().await {
            if let Ok(notification) = notification {
                let producer = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = producer.process_notification(notification).await {
                        error!("Error processing notification: {:?}", e);
                    }
                });
            }
        }

        error!("Changing data capture end");
        Ok(())
    }

    async fn process_notification(&self, notification: PgNotification) -> Result<()> {
        let payload: Value = serde_json::from_str(&notification.payload())?;
        let event = self.parse_event(notification.channel(), payload)?;
        info!("process_notification Event: {:?}", event);
        let messages = self.handle_order_event(event).await?;
        self.broadcast_messages(messages).await?;

        Ok(())
    }

    fn parse_event(&self, channel: &str, payload: Value) -> Result<OrderEventCapture> {
        info!("parse_event channel = {:?}", channel);
        match channel {
            TOKEN => Ok(OrderEventCapture::CreationTime(Token::from_value(payload)?)),
            SWAP => Ok(OrderEventCapture::BumpOrder(Swap::from_value(payload)?)),
            CURVE => Ok(OrderEventCapture::MartKetCap(Curve::from_value(payload)?)),
            TOKEN_REPLIES_COUNT => Ok(OrderEventCapture::ReplyChange(TokenReplyCount::from_value(
                payload,
            )?)),

            _ => Err(anyhow::anyhow!("Unknown channel: {}", channel)),
        }
    }

    async fn broadcast_messages(&self, messages: Vec<OrderMessage>) -> Result<()> {
        let senders = self.order_senders.read().await;
        for message in messages {
            // 메시지의 order type 가져오기
            let order_type = message.order_type;

            match senders.get(&order_type) {
                Some(sender) => {
                    if let Err(e) = sender.0.send(message) {
                        warn!(
                            "OrderType {:?}에 대한 메시지 전송 실패: {:?}",
                            order_type, e
                        );
                    }
                }
                None => {
                    warn!(
                        "OrderType {:?}에 대한 sender를 찾을 수 없습니다",
                        order_type
                    );
                }
            }
        }
        Ok(())
    }

    async fn handle_order_event(&self, event: OrderEventCapture) -> Result<Vec<OrderMessage>> {
        match event {
            OrderEventCapture::CreationTime(token) => self.handle_creation_time_order(token).await,
            OrderEventCapture::BumpOrder(swap) => self.handle_bump_order(swap).await,
            OrderEventCapture::MartKetCap(curve) => self.handle_market_cap_order(curve).await,
            OrderEventCapture::ReplyChange(token_reply) => {
                self.handle_reply_change_order(token_reply).await
            }
        }
    }

    //Handle redis save
    async fn add_creation_time_order(
        &self,
        db: Arc<PostgresDatabase>,
        token: Token,
    ) -> Result<Option<OrderTokenResponse>> {
        info!("Add_creation_time_order start");
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_token(&token.id)
            .await?;
        let result = self
            .redis
            .add_to_creation_time_order(&order_repsonse, token.created_at.to_string())
            .await
            .context("Add_to_creation_time_order fail")?;
        info!("Add_to_creation_time_order success");
        Ok(if result { Some(order_repsonse) } else { None })
    }

    async fn add_bump_order(
        &self,
        db: Arc<PostgresDatabase>,
        swap: Swap,
    ) -> Result<Option<OrderTokenResponse>> {
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_token(&swap.token_id)
            .await?;
        let result = self
            .redis
            .add_to_bump_order(&order_repsonse, swap.created_at.to_string())
            .await
            .context("Add_to_bump_order fail")?;

        Ok(if result { Some(order_repsonse) } else { None })
    }
    async fn add_market_cap_order(
        &self,
        db: Arc<PostgresDatabase>,
        curve: Curve,
    ) -> Result<Option<OrderTokenResponse>> {
        // let token = token_controller.get_token_by_id(&curve.token_id).await?;
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_token(&curve.token_id)
            .await?;
        let score = curve.price.to_string();
        let result = self
            .redis
            .add_to_market_cap_order(&order_repsonse, score)
            .await?;
        Ok(if result { Some(order_repsonse) } else { None })
    }

    async fn add_token_last_reply_order(
        &self,
        db: Arc<PostgresDatabase>,
        token_reply: TokenReplyCount,
    ) -> Result<Option<OrderTokenResponse>> {
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_token(&token_reply.token_id)
            .await?;
        let score = order_repsonse.created_at.to_string();
        let redis_result = self
            .redis
            .add_to_last_reply_order(&order_repsonse, score.to_string())
            .await?;
        Ok(if redis_result {
            Some(order_repsonse)
        } else {
            None
        })
    }
    async fn add_token_reply_count_order(
        &self,
        db: Arc<PostgresDatabase>,
        token_reply: TokenReplyCount,
    ) -> Result<Option<OrderTokenResponse>> {
        let order_controller = OrderController::new(db);
        let token = order_controller
            .get_order_token_response_by_token(&token_reply.token_id)
            .await?;
        let result = self
            .redis
            .add_to_reply_count_order(&token, token_reply.reply_count.to_string())
            .await?;
        Ok(if result { Some(token) } else { None })
    }

    async fn handle_creation_time_order(&self, token: Token) -> Result<Vec<OrderMessage>> {
        info!("Handle_creation_time_order start");
        let order_token_response = self
            .add_creation_time_order(self.db.clone(), token)
            .await
            .context("Fail add_bump_order")?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to add token, which should never happen for creation_time order"
                )
            })?;

        let message = OrderMessage {
            order_type: OrderType::CreationTime,
            order_token: Some(vec![order_token_response]),
        };
        info!("Handle_creation_time_order success");
        Ok(vec![message])
    }

    async fn handle_bump_order(&self, swap: Swap) -> Result<Vec<OrderMessage>> {
        let order_token_response = self
            .add_bump_order(self.db.clone(), swap.clone())
            .await
            .context("Fail add_bump_order")?
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to add token, which should never happen for bump order")
            })?;

        let message = OrderMessage {
            order_type: OrderType::Bump,
            order_token: Some(vec![order_token_response.clone()]),
        };

        Ok(vec![message])
    }

    async fn handle_market_cap_order(&self, curve: Curve) -> Result<Vec<OrderMessage>> {
        match self
            .add_market_cap_order(self.db.clone(), curve.clone())
            .await?
        {
            Some(token) => {
                info!("Market cap order added successfully");
                let message = OrderMessage {
                    order_type: OrderType::MarketCap,
                    order_token: Some(vec![token]),
                };
                Ok(vec![message])
            }
            None => Ok(vec![]),
        }
    }

    async fn handle_reply_change_order(
        &self,
        token_reply: TokenReplyCount,
    ) -> Result<Vec<OrderMessage>> {
        info!("Handle_reply_change_order start");
        let lastest_reply_result = self
            .add_token_last_reply_order(self.db.clone(), token_reply.clone())
            .await?;
        let reply_count_result = self
            .add_token_reply_count_order(self.db.clone(), token_reply.clone())
            .await?;
        let mut messages = Vec::new();

        if let Some(last_reply_token) = lastest_reply_result {
            messages.push(OrderMessage {
                // message_type: SendMessageType::Regular,
                // new_token: None,
                // new_buy: None,
                // new_sell: None,
                order_type: OrderType::LatestReply,
                order_token: Some(vec![last_reply_token]),
            });
        }

        if let Some(reply_count_token) = reply_count_result {
            messages.push(OrderMessage {
                // message_type: SendMessageType::Regular,
                // new_token: None,
                // new_buy: None,
                // new_sell: None,
                order_type: OrderType::ReplyCount,
                order_token: Some(vec![reply_count_token]),
            });
        }

        Ok(messages)
    }

    pub async fn get_order_receiver(&self, order_type: OrderType) -> OrderReceiver {
        let mut senders = self.order_senders.write().await;
        let (sender, count) = senders.entry(order_type).or_insert_with(|| {
            self.total_channels
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            (broadcast::channel(100).0, 0)
        });

        *count += 1;
        info!(
            "Increment order receiver count ={:?} order_type = {:?}",
            *count, order_type
        );

        OrderReceiver {
            receiver: sender.subscribe(),
            order_type,
            controller: self.clone().into(),
        }
    }
    async fn decrement_receiver_count(&self, order_type: OrderType) {
        let mut senders = self.order_senders.write().await;
        if let Some((_, count)) = senders.get_mut(&order_type) {
            *count -= 1;
            info!("Decrement order receiver count ={:?}", *count);
            if *count == 0 {
                senders.remove(&order_type);
                self.total_channels.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
}

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    constant::change_channels::{COIN, COIN_REPLIES_COUNT, CURVE, SWAP},
    db::{
        postgres::{
            controller::{info::InfoController, order::OrderController},
            PostgresDatabase,
        },
        redis::RedisDatabase,
    },
    types::{
        event::{
            order::{OrderEvent, OrderMessage, OrderTokenResponse, OrderType},
            NewSwapMessage, NewTokenMessage, SendMessageType,
        },
        model::{
            Coin, CoinReplyCount, CoinReplyCountWrapper, CoinWrapper, Curve, CurveWrapper,
            FromValue, Swap, SwapWrapper,
        },
    },
};
use anyhow::{Context, Result};
use chrono::Utc;
use futures::{future::try_join_all, StreamExt};

use serde_json::Value;
use sqlx::postgres::PgListener;
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
        let coins = match order_type {
            OrderType::CreationTime => controller.get_creation_time_order_token().await?,
            OrderType::MarketCap => controller.get_market_cap_order_token().await?,
            OrderType::Bump => controller.get_bump_order_token().await?,
            OrderType::ReplyCount => controller.get_reply_count_order_token().await?,
            OrderType::LatestReply => controller.get_latest_reply_order_token().await?,
        };
        // info!("Fetched coins for {:?}", coins);
        match order_type {
            OrderType::CreationTime => self.redis.set_creation_time_order(coins).await?,
            OrderType::MarketCap => self.redis.set_market_cap_order(coins).await?,
            OrderType::Bump => self.redis.set_bump_order(coins).await?,
            OrderType::ReplyCount => self.redis.set_reply_count_order(coins).await?,
            OrderType::LatestReply => self.redis.set_last_reply_order(coins).await?,
        }

        // info!("Set event for {:?}", order_type);
        Ok(())
    }
    #[instrument(skip(self))]
    pub async fn change_data_capture(&self) -> Result<()> {
        let mut listener = PgListener::connect_with(&self.db.pool).await?;
        listener
            .listen_all(vec![COIN, SWAP, CURVE, COIN_REPLIES_COUNT])
            .await?;
        let mut stream = listener.into_stream();
        info!("Order event  capture Start");

        while let Some(notification) = stream.next().await {
            if let Ok(notification) = notification {
                // info!("Notification {:?}", notification);
                if self.total_channels.load(Ordering::Relaxed) == 0 {
                    continue; // 총 채널 수가 0이면 다음 알림으로 넘어갑니다.
                }
                info!("Changing data capture start");
                let payload: Value = serde_json::from_str(&notification.payload())?;

                let senders = self.order_senders.read().await;

                let event = match notification.channel() {
                    COIN => OrderEvent::CreationTime(Coin::from_value(payload)?),
                    SWAP => OrderEvent::BumpOrder(Swap::from_value(payload)?),
                    CURVE => OrderEvent::MartKetCap(Curve::from_value(payload)?),
                    COIN_REPLIES_COUNT => {
                        OrderEvent::ReplyChange(CoinReplyCount::from_value(payload)?)
                    }
                    _ => continue,
                };

                let handle_result = self.handle_order_event(event).await;
                let messages = match handle_result {
                    Ok(messages) => {
                        debug!("Handled event successfully");
                        messages
                    }
                    Err(e) => {
                        warn!("Failed to handle event: {:?}", e);
                        vec![]
                    }
                };

                for message in messages {
                    match message.message_type {
                        SendMessageType::ALL => {
                            // Broadcast to all connected users
                            for (_, sender) in senders.iter() {
                                debug!("Sending message to all");
                                let _ = sender.0.send(message.clone());
                            }
                        }
                        SendMessageType::Regular => {
                            // Send only to subscribers of this coin
                            if let Some(sender) = senders.get(&message.order_type) {
                                let _ = sender.0.send(message);
                            }
                        }
                    }
                }
            }
        }
        error!("Changing data capture end");
        Ok(())
    }

    //Handle redis save
    async fn add_creation_time_order(
        &self,
        db: Arc<PostgresDatabase>,
        coin: Coin,
    ) -> Result<Option<OrderTokenResponse>> {
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_coin_id(&coin.id)
            .await?;
        let result = self
            .redis
            .add_to_creation_time_order(&order_repsonse, coin.created_at.to_string())
            .await
            .context("Add_to_creation_time_order fail")?;

        Ok(if result { Some(order_repsonse) } else { None })
    }

    async fn add_bump_order(
        &self,
        db: Arc<PostgresDatabase>,
        swap: Swap,
    ) -> Result<Option<OrderTokenResponse>> {
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_coin_id(&swap.coin_id)
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
        // let coin = coin_controller.get_coin_by_id(&curve.coin_id).await?;
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_coin_id(&curve.coin_id)
            .await?;
        let score = curve.price.to_string();
        let result = self
            .redis
            .add_to_market_cap_order(&order_repsonse, score)
            .await?;
        Ok(if result { Some(order_repsonse) } else { None })
    }

    async fn add_coin_last_reply_order(
        &self,
        db: Arc<PostgresDatabase>,
        coin_reply: CoinReplyCount,
    ) -> Result<Option<OrderTokenResponse>> {
        let order_controller = OrderController::new(db);
        let order_repsonse = order_controller
            .get_order_token_response_by_coin_id(&coin_reply.coin_id)
            .await?;
        let score = Utc::now().timestamp();
        let result = self
            .redis
            .add_to_last_reply_order(&order_repsonse, score.to_string())
            .await?;
        Ok(if result { Some(order_repsonse) } else { None })
    }
    async fn add_coin_reply_count_order(
        &self,
        db: Arc<PostgresDatabase>,
        coin_reply: CoinReplyCount,
    ) -> Result<Option<OrderTokenResponse>> {
        let order_controller = OrderController::new(db);
        let coin = order_controller
            .get_order_token_response_by_coin_id(&coin_reply.coin_id)
            .await?;
        let result = self
            .redis
            .add_to_reply_count_order(&coin, coin_reply.reply_count.to_string())
            .await?;
        Ok(if result { Some(coin) } else { None })
    }

    // Handle message

    async fn handle_order_event(&self, event: OrderEvent) -> Result<Vec<OrderMessage>> {
        match event {
            OrderEvent::CreationTime(coin) => {
                debug!("Coin creation time event {:?}", coin);

                Ok(self.handle_creation_time_order(coin).await?)
            }
            OrderEvent::BumpOrder(swap) => {
                debug!("Swap bump order event {:?}", swap);

                Ok(self.handle_bump_order(swap).await?)
            }
            OrderEvent::MartKetCap(curve) => {
                debug!("Curve market cap event {:?}", curve);
                Ok(self.handle_market_cap_order(curve).await?)
            }
            OrderEvent::ReplyChange(coin_reply) => {
                debug!("Coin latest reply count event {:?}", coin_reply);
                Ok(self.handle_reply_change_order(coin_reply).await?)
            }
        }
    }

    async fn handle_creation_time_order(&self, coin: Coin) -> Result<Vec<OrderMessage>> {
        let order_token_response = self
            .add_creation_time_order(self.db.clone(), coin)
            .await
            .context("Fail add_bump_order")?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to add coin, which should never happen for creation_time order"
                )
            })?;

        let new_token = NewTokenMessage::new(&order_token_response);

        self.redis
            .set_new_token(&new_token)
            .await
            .context("Failed Set New Token")?;

        let message = OrderMessage {
            message_type: SendMessageType::ALL,
            new_token: Some(new_token),
            new_buy: None,
            new_sell: None,
            order_type: OrderType::CreationTime,
            order_token: Some(vec![order_token_response]),
        };
        Ok(vec![message])
    }

    async fn handle_bump_order(&self, swap: Swap) -> Result<Vec<OrderMessage>> {
        //timestamp 때문에 그렇나?
        let order_token_response = self
            .add_bump_order(self.db.clone(), swap.clone())
            .await
            .context("Fail add_bump_order")?
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to add coin, which should never happen for bump order")
            })?;

        let info_controller = InfoController::new(self.db.clone());
        let trader_info = info_controller.get_user(&swap.sender).await?;

        let new_swap_message = NewSwapMessage::new(&order_token_response, trader_info, &swap);
        self.redis
            .set_new_swap(&new_swap_message)
            .await
            .context("Failed Set New Swap")?;
        let message = match new_swap_message.is_buy {
            true => {
                let message = OrderMessage {
                    message_type: SendMessageType::ALL,
                    new_token: None,
                    new_buy: Some(new_swap_message),
                    new_sell: None,

                    order_type: OrderType::Bump,
                    order_token: Some(vec![order_token_response.clone()]),
                };
                message
            }
            false => {
                let message = OrderMessage {
                    message_type: SendMessageType::ALL,
                    new_token: None,
                    new_buy: None,
                    new_sell: Some(new_swap_message),
                    // new_swap: Some(new_swap_message),
                    order_type: OrderType::Bump,
                    order_token: Some(vec![order_token_response.clone()]),
                };
                message
            }
        };

        Ok(vec![message])
    }

    async fn handle_market_cap_order(&self, curve: Curve) -> Result<Vec<OrderMessage>> {
        match self
            .add_market_cap_order(self.db.clone(), curve.clone())
            .await?
        {
            Some(coin) => {
                info!("Market cap order added successfully");
                let message = OrderMessage {
                    message_type: SendMessageType::Regular,
                    new_token: None,
                    new_buy: None,
                    new_sell: None,
                    order_type: OrderType::MarketCap,
                    order_token: Some(vec![coin]),
                };
                Ok(vec![message])
            }
            None => Ok(vec![]),
        }
    }

    async fn handle_reply_change_order(
        &self,
        coin_reply: CoinReplyCount,
    ) -> Result<Vec<OrderMessage>> {
        let last_reply_result = self
            .add_coin_last_reply_order(self.db.clone(), coin_reply.clone())
            .await?;
        let reply_count_result = self
            .add_coin_reply_count_order(self.db.clone(), coin_reply.clone())
            .await?;

        let mut messages = Vec::new();

        if let Some(last_reply_coin) = last_reply_result {
            messages.push(OrderMessage {
                message_type: SendMessageType::Regular,
                new_token: None,
                new_buy: None,
                new_sell: None,
                order_type: OrderType::LatestReply,
                order_token: Some(vec![last_reply_coin]),
            });
        }

        if let Some(reply_count_coin) = reply_count_result {
            messages.push(OrderMessage {
                message_type: SendMessageType::Regular,
                new_token: None,
                new_buy: None,
                new_sell: None,
                order_type: OrderType::ReplyCount,
                order_token: Some(vec![reply_count_coin]),
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
        info!("Increment order receiver count ={:?}", *count);
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

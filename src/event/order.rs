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
            controller::{message::MessageController, order::OrderController},
            PostgresDatabase,
        },
        redis::RedisDatabase,
    },
    types::{
        event::{
            order::{
                CreateCoinMessage, CreateSwapCoinInfo, CreateSwapMesage, OrderCoinResponse,
                OrderEvent, OrderMessage, OrderType,
            },
            wrapper::{CoinWrapper, CurveWrapper, ReplyCountWrapper, SwapWrapper},
            SendMessageType,
        },
        model::{Coin, CoinReplyCount, Curve, Swap},
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
use tracing::{error, info, instrument, warn};

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
            OrderType::CreationTime => controller.get_creation_time_order_coin().await?,
            OrderType::MarketCap => controller.get_market_cap_order_coin().await?,
            OrderType::Bump => controller.get_bump_order_coin().await?,
            OrderType::ReplyCount => controller.get_reply_count_order_coin().await?,
            OrderType::LatestReply => controller.get_latest_reply_order_coin().await?,
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

                info!("channel {:?}", notification.channel());
                let event = match notification.channel() {
                    COIN => OrderEvent::CreationTime(CoinWrapper::parse(payload)?),
                    SWAP => OrderEvent::BumpOrder(SwapWrapper::parse(payload)?),
                    CURVE => OrderEvent::MartKetCap(CurveWrapper::parse(payload)?),
                    COIN_REPLIES_COUNT => {
                        OrderEvent::ReplyChange(ReplyCountWrapper::parse(payload)?)
                    }
                    _ => continue,
                };
                info!("Event {:?}", event);
                let handle_result = self.handle_order_event(event).await;
                let messages = match handle_result {
                    Ok(messages) => {
                        info!("Messages {:?}", messages);
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
                                info!("Sending message to all");
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
    ) -> Result<Option<OrderCoinResponse>> {
        let order_controller = OrderController::new(db);
        let coin = order_controller.get_order_coin_response(&coin.id).await?;
        let result = self
            .redis
            .add_to_creation_time_order(&coin, coin.created_at.to_string())
            .await
            .context("Add_to_creation_time_order fail")?;
        Ok(if result { Some(coin) } else { None })
    }

    async fn add_bump_order(
        &self,
        db: Arc<PostgresDatabase>,
        swap: Swap,
    ) -> Result<Option<OrderCoinResponse>> {
        let order_controller = OrderController::new(db);
        let coin = order_controller
            .get_order_coin_response(&swap.coin_id)
            .await?;
        let result = self
            .redis
            .add_to_bump_order(&coin, swap.created_at.to_string())
            .await
            .context("Add_to_bump_order fail")?;

        Ok(if result { Some(coin) } else { None })
    }
    async fn add_market_cap_order(
        &self,
        db: Arc<PostgresDatabase>,
        curve: Curve,
    ) -> Result<Option<OrderCoinResponse>> {
        // let coin = coin_controller.get_coin_by_id(&curve.coin_id).await?;
        let order_controller = OrderController::new(db);
        let coin = order_controller
            .get_order_coin_response(&curve.coin_id)
            .await?;
        let score = curve.price.to_string();
        let result = self.redis.add_to_market_cap_order(&coin, score).await?;
        Ok(if result { Some(coin) } else { None })
    }

    async fn add_coin_last_reply_order(
        &self,
        db: Arc<PostgresDatabase>,
        coin_reply: CoinReplyCount,
    ) -> Result<Option<OrderCoinResponse>> {
        let order_controller = OrderController::new(db);
        let coin = order_controller
            .get_order_coin_response(&coin_reply.coin_id)
            .await?;
        let score = Utc::now().timestamp();
        let result = self
            .redis
            .add_to_last_reply_order(&coin, score.to_string())
            .await?;
        Ok(if result { Some(coin) } else { None })
    }
    async fn add_coin_reply_count_order(
        &self,
        db: Arc<PostgresDatabase>,
        coin_reply: CoinReplyCount,
    ) -> Result<Option<OrderCoinResponse>> {
        let order_controller = OrderController::new(db);
        let coin = order_controller
            .get_order_coin_response(&coin_reply.coin_id)
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
                info!("Coin creation time event {:?}", coin);

                Ok(self.handle_creation_time_order(coin).await?)
            }
            OrderEvent::BumpOrder(swap) => {
                info!("Swap bump order event {:?}", swap);

                Ok(self.handle_bump_order(swap).await?)
            }
            OrderEvent::MartKetCap(curve) => {
                info!("Curve market cap event {:?}", curve);
                Ok(self.handle_market_cap_order(curve).await?)
            }
            OrderEvent::ReplyChange(coin_reply) => {
                info!("Coin latest reply count event {:?}", coin_reply);
                Ok(self.handle_reply_change_order(coin_reply).await?)
            }
        }
    }

    async fn handle_creation_time_order(&self, coin: Coin) -> Result<Vec<OrderMessage>> {
        let coin = self
            .add_creation_time_order(self.db.clone(), coin)
            .await
            .context("Fail add_bump_order")?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to add coin, which should never happen for creation_time order"
                )
            })?;

        let creator = coin.creator.clone();
        let message = OrderMessage {
            message_type: SendMessageType::ALL,
            new_token: Some(CreateCoinMessage {
                creator,
                symbol: coin.symbol.clone(),
                image_uri: coin.image_uri.clone(),
                created_at: coin.created_at,
            }),
            new_swap: None,
            order_type: OrderType::CreationTime,
            order_coin: Some(vec![coin]),
        };
        Ok(vec![message])
    }

    async fn handle_bump_order(&self, swap: Swap) -> Result<Vec<OrderMessage>> {
        info!("handle_bump_order swap {:?}", swap);

        //timestamp 때문에 그렇나?
        let coin = self
            .add_bump_order(self.db.clone(), swap.clone())
            .await
            .context("Fail add_bump_order")?
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to add coin, which should never happen for bump order")
            })?;

        info!("handle_bump_order coin {:?}", coin);
        let message_controller = MessageController::new(self.db.clone());
        let trader = message_controller.get_user(&swap.sender).await?;
        info!("trader  {:?}", trader);
        let coin_info = CreateSwapCoinInfo {
            symbol: coin.symbol.clone(),
            image_uri: coin.image_uri.clone(),
        };

        let create_swap_message = CreateSwapMesage {
            trader_info: trader,
            is_buy: swap.is_buy,
            nad_amount: swap.nad_amount.to_string(),
            coin_info,
        };
        let message = OrderMessage {
            message_type: SendMessageType::ALL,
            new_token: None,
            new_swap: Some(create_swap_message),
            order_type: OrderType::Bump,
            order_coin: Some(vec![coin.clone()]),
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
                    new_swap: None,
                    order_type: OrderType::MarketCap,
                    order_coin: Some(vec![coin]),
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
                new_swap: None,
                order_type: OrderType::LatestReply,
                order_coin: Some(vec![last_reply_coin]),
            });
        }

        if let Some(reply_count_coin) = reply_count_result {
            messages.push(OrderMessage {
                message_type: SendMessageType::Regular,
                new_token: None,
                new_swap: None,
                order_type: OrderType::ReplyCount,
                order_coin: Some(vec![reply_count_coin]),
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

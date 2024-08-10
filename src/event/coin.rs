use crate::{
    constant::change_channels::{BALANCE, CHART, COIN, CURVE, SWAP, THREAD},
    db::{
        postgres::{controller::info::InfoController, PostgresDatabase},
        redis::RedisDatabase,
    },
    types::{
        event::{
            coin_message::CoinMessage,
            wrapper::{
                BalanceWrapper, ChartWrapper, CoinWrapper, CurveWrapper, SwapWrapper, ThreadWrapper,
            },
            SendMessageType,
        },
        model::{Balance, Chart, Coin, Curve, Swap, Thread},
    },
};
use anyhow::Result;
use futures::{StreamExt, TryFutureExt};
use serde_json::Value;
use sqlx::postgres::PgListener;
use std::{collections::HashMap, sync::atomic::AtomicUsize};
use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use tokio::{
    sync::{
        broadcast::{self, error::RecvError, Receiver, Sender},
        RwLock,
    },
    time::sleep,
};
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip(producer))]
pub async fn main(producer: Arc<CoinEventProducer>) -> Result<()> {
    info!("Starting coin event capture");

    loop {
        match producer.change_data_capture().await {
            Ok(_) => {
                warn!("Coin event capture completed unexpectedly");
                break;
            }
            Err(e) => {
                error!("Error in coin change_data_capture: {:?}", e);
                info!("Retrying in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    error!("Coin event capture ended");
    Ok(())
}
#[derive(Clone, Debug)]
pub enum CoinEvent {
    Coin(Coin),
    Swap(Swap),
    Chart(ChartWrapper),
    Balance(Balance),
    Curve(Curve),
    Thread(Thread),
}

pub struct CoinReceiver {
    receiver: Receiver<CoinMessage>,
    coin_id: String,
    controller: Arc<CoinEventProducer>,
}
impl CoinReceiver {
    pub async fn recv(&mut self) -> Result<CoinMessage, RecvError> {
        self.receiver.recv().await
    }
}

impl Drop for CoinReceiver {
    fn drop(&mut self) {
        let controller = self.controller.clone();
        let coin_id = self.coin_id.clone();
        tokio::spawn(async move {
            controller.decrement_receiver_count(&coin_id).await;
        });
    }
}

#[derive(Clone)]
pub struct CoinEventProducer {
    db: Arc<PostgresDatabase>,
    coin_senders: Arc<RwLock<HashMap<String, (Sender<CoinMessage>, usize)>>>,
    total_channels: Arc<AtomicUsize>,
}

impl CoinEventProducer {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        CoinEventProducer {
            db,
            coin_senders: Arc::new(RwLock::new(HashMap::new())),
            total_channels: Arc::new(AtomicUsize::new(0)),
        }
    }
    #[instrument(skip(self))]
    pub async fn change_data_capture(&self) -> Result<()> {
        let mut listener = PgListener::connect_with(&self.db.pool).await?;
        listener
            .listen_all(vec![COIN, SWAP, CHART, BALANCE, CURVE, THREAD])
            .await?;
        let mut stream = listener.into_stream();
        info!("Coin event capture started");
        while let Some(notification) = stream.next().await {
            if let Ok(notification) = notification {
                if self.total_channels.load(Ordering::Relaxed) == 0 {
                    continue;
                }

                let payload: Value = serde_json::from_str(&notification.payload())?;
                let coin_id = payload["coin_id"].as_str().unwrap_or("").to_string();

                let event = match notification.channel() {
                    COIN => CoinEvent::Coin(CoinWrapper::parse(payload)?),
                    SWAP => CoinEvent::Swap(SwapWrapper::parse(payload)?),
                    CHART => CoinEvent::Chart(ChartWrapper::parse(payload)?),
                    BALANCE => CoinEvent::Balance(BalanceWrapper::parse(payload)?),
                    CURVE => CoinEvent::Curve(CurveWrapper::parse(payload)?),
                    THREAD => CoinEvent::Thread(ThreadWrapper::parse(payload)?),
                    _ => continue,
                };

                let senders = self.coin_senders.read().await;
                let should_process = match &event {
                    CoinEvent::Coin(_) | CoinEvent::Swap(_) => true,
                    _ => senders.contains_key(&coin_id),
                };

                if should_process {
                    let messages = self.handle_event(event).await?;
                    info!("Message length is {:?}", messages.len());
                    for message in messages {
                        info!(
                            "message type = {:?} coin_id = {:?}",
                            message.message_type, message.coin.id
                        );
                        match message.message_type {
                            SendMessageType::ALL => {
                                for (_, sender) in senders.iter() {
                                    match sender.0.send(message.clone()) {
                                        Ok(_) => warn!("Message All sent successfully"),
                                        Err(e) => warn!("Failed to send message: {:?}", e),
                                    }
                                }
                            }
                            SendMessageType::Regular => {
                                if let Some(sender) = senders.get(&message.coin.id) {
                                    info!(
                                        "Sending Regular message for coin_id: {}",
                                        message.coin.id
                                    );
                                    match sender.0.send(message.clone()) {
                                        Ok(_) => info!(
                                            "Regular message sent successfully for coin_id: {:?}",
                                            message
                                        ),
                                        Err(e) => warn!(
                                            "Failed to send Regular message for coin_id: {}: {:?}",
                                            message.coin.id, e
                                        ),
                                    }
                                } else {
                                    warn!("No sender found for coin_id: {}", message.coin.id);
                                }
                            }
                        }
                    }
                } else {
                    info!("Skipping event for coin_id: {}, no subscribers", coin_id);
                }
            }
        }
        Ok(())
    }

    async fn handle_event(&self, event: CoinEvent) -> Result<Vec<CoinMessage>> {
        let result = match event {
            CoinEvent::Coin(coin) => self.handle_coin_event(coin).map_ok(|m| m).await,
            CoinEvent::Swap(swap) => self.handle_swap_event(swap).map_ok(|m| m).await,
            CoinEvent::Chart(chart) => self.handle_chart_event(chart).map_ok(|m| m).await,
            CoinEvent::Balance(balance) => self.handle_balance_event(balance).map_ok(|m| m).await,
            CoinEvent::Curve(curve) => self.handle_curve_event(curve).map_ok(|m| m).await,
            CoinEvent::Thread(thread) => self.handle_thread_event(thread).map_ok(|m| m).await,
        };

        match result {
            Ok(messages) => {
                info!("Successfully handled {} event(s)", messages.len());
                Ok(messages)
            }
            Err(e) => {
                error!("Error handling event: {:?}", e);
                Err(e)
            }
        }
    }
    async fn handle_coin_event(&self, coin: Coin) -> Result<Vec<CoinMessage>> {
        let info_controller = InfoController::new(self.db.clone());
        let info = info_controller
            .get_coin_and_user_info(&coin.id, &coin.creator)
            .await?;

        Ok(vec![CoinMessage::from_coin(coin, info)])
    }
    async fn handle_swap_event(&self, swap: Swap) -> Result<Vec<CoinMessage>> {
        let info_controller = InfoController::new(self.db.clone());
        let info = info_controller
            .get_coin_and_user_info(&swap.coin_id, &swap.sender)
            .await?;

        Ok(vec![CoinMessage::from_swap(swap, info)])
    }

    async fn handle_chart_event(&self, chart: ChartWrapper) -> Result<Vec<CoinMessage>> {
        Ok(vec![CoinMessage::from_chart(chart)])
    }

    async fn handle_balance_event(&self, balance: Balance) -> Result<Vec<CoinMessage>> {
        Ok(vec![CoinMessage::from_balance(balance)])
    }

    async fn handle_curve_event(&self, curve: Curve) -> Result<Vec<CoinMessage>> {
        Ok(vec![CoinMessage::from_curve(curve)])
    }

    async fn handle_thread_event(&self, thread: Thread) -> Result<Vec<CoinMessage>> {
        // info!("Handling thread event {:?}", thread);
        // Thread 이벤트 처리 로직
        Ok(vec![CoinMessage::from_thread(thread)])
    }
    pub async fn get_coin_receiver(&self, coin_id: &str) -> CoinReceiver {
        let mut senders = self.coin_senders.write().await;
        let (sender, count) = senders.entry(coin_id.to_string()).or_insert_with(|| {
            let new_count = self.total_channels.fetch_add(1, Ordering::SeqCst) + 1;
            info!(
                "Creating new channel for coin_id: {}. Total channels: {}",
                coin_id, new_count
            );
            (broadcast::channel(1000).0, 0)
        });

        *count += 1;
        info!(
            "Incrementing count for coin_id: {}. New count: {}. Total channels: {}",
            coin_id,
            count,
            self.total_channels.load(Ordering::SeqCst)
        );

        let receiver = sender.subscribe();
        info!("Created new receiver for coin_id: {}", coin_id);

        CoinReceiver {
            receiver,
            coin_id: coin_id.to_string(),
            controller: self.clone().into(),
        }
    }
    async fn decrement_receiver_count(&self, coin_id: &str) {
        let mut senders = self.coin_senders.write().await;
        if let Some((_, count)) = senders.get_mut(coin_id) {
            *count -= 1;
            info!(
                "Decremented count for coin_id: {}. New count: {}",
                coin_id, count
            );
            if *count == 0 {
                senders.remove(coin_id);
                info!("Removed channel for coin_id: {}", coin_id);
            }
        }
        self.total_channels.fetch_sub(1, Ordering::SeqCst);
    }
}

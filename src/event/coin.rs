use crate::{
    constant::change_channels::{BALANCE, CHART, COIN, CURVE, SWAP, THREAD},
    db::{
        postgres::{controller::message::MessageController, PostgresDatabase},
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
use futures::StreamExt;
use serde_json::Value;
use sqlx::postgres::PgListener;
use std::{collections::HashMap, sync::atomic::AtomicUsize};
use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use tokio::{
    sync::{
        broadcast::{self, Receiver, Sender},
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
    Chart(Chart),
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
    pub async fn recv(&mut self) -> Option<CoinMessage> {
        self.receiver.recv().await.ok()
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
                //총 채널이 0이라면 다음 loop 넘어감.
                if self.total_channels.load(Ordering::Relaxed) == 0 {
                    continue;
                }

                let payload: Value = serde_json::from_str(&notification.payload())?;

                let senders = self.coin_senders.read().await;
                let coin_id = payload["coin_id"].to_string();
                let event = match notification.channel() {
                    COIN => CoinEvent::Coin(CoinWrapper::parse(payload)?),
                    SWAP => CoinEvent::Swap(SwapWrapper::parse(payload)?),
                    CHART => CoinEvent::Chart(ChartWrapper::parse(payload)?),
                    BALANCE => CoinEvent::Balance(BalanceWrapper::parse(payload)?),
                    CURVE => CoinEvent::Curve(CurveWrapper::parse(payload)?),
                    THREAD => CoinEvent::Thread(ThreadWrapper::parse(payload)?),
                    _ => continue,
                };

                //event 가 coin,swap 이 아니고 coin_id 에 맞는 channel 이 없으면 continue
                let should_continue = match &event {
                    CoinEvent::Coin(_) | CoinEvent::Swap(_) => false,
                    _ => !senders.contains_key(&coin_id),
                };

                if should_continue {
                    continue;
                }

                let messages = self.handle_event(event).await?;
                debug!("Coin event {:?}", messages);
                for message in messages {
                    match message.message_type {
                        SendMessageType::ALL => {
                            // Broadcast to all connected users
                            for (_, sender) in senders.iter() {
                                let _ = sender.0.send(message.clone());
                            }
                        }
                        SendMessageType::Regular => {
                            // Send only to subscribers of this coin
                            if let Some(sender) = senders.get(&message.coin.id) {
                                let _ = sender.0.send(message);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_event(&self, event: CoinEvent) -> Result<Vec<CoinMessage>> {
        info!("Handling event {:?}", event);
        match event {
            CoinEvent::Coin(coin) => self.handle_coin_event(coin).await,
            CoinEvent::Swap(swap) => self.handle_swap_event(swap).await,
            CoinEvent::Chart(chart) => self.handle_chart_event(chart).await,
            CoinEvent::Balance(balance) => self.handle_balance_event(balance).await,
            CoinEvent::Curve(curve) => self.handle_curve_event(curve).await,
            CoinEvent::Thread(thread) => self.handle_thread_event(thread).await,
        }
    }
    async fn handle_coin_event(&self, coin: Coin) -> Result<Vec<CoinMessage>> {
        let message_controller = MessageController::new(self.db.clone());
        let info = message_controller
            .get_coin_and_user_info(&coin.id, &coin.creator)
            .await?;

        Ok(vec![CoinMessage::from_coin(coin, info)])
    }
    async fn handle_swap_event(&self, swap: Swap) -> Result<Vec<CoinMessage>> {
        let message_controller = MessageController::new(self.db.clone());
        let info = message_controller
            .get_coin_and_user_info(&swap.coin_id, &swap.sender)
            .await?;

        Ok(vec![CoinMessage::from_swap(swap, info)])
    }

    async fn handle_chart_event(&self, chart: Chart) -> Result<Vec<CoinMessage>> {
        // Chart 이벤트 처리 로직
        Ok(vec![CoinMessage::from_chart(chart)])
    }

    async fn handle_balance_event(&self, balance: Balance) -> Result<Vec<CoinMessage>> {
        // Balance 이벤트 처리 로직
        Ok(vec![CoinMessage::from_balance(balance)])
    }

    async fn handle_curve_event(&self, curve: Curve) -> Result<Vec<CoinMessage>> {
        // Curve 이벤트 처리 로직
        Ok(vec![CoinMessage::from_curve(curve)])
    }

    async fn handle_thread_event(&self, thread: Thread) -> Result<Vec<CoinMessage>> {
        // Thread 이벤트 처리 로직
        Ok(vec![CoinMessage::from_thread(thread)])
    }
    pub async fn get_coin_receiver(&self, coin_id: &str) -> CoinReceiver {
        let mut senders = self.coin_senders.write().await;
        let (sender, count) = senders.entry(coin_id.to_string()).or_insert_with(|| {
            self.total_channels.fetch_add(1, Ordering::SeqCst);
            (broadcast::channel(100).0, 0)
        });

        *count += 1;
        info!(
            "Incrementing count for coin_id: {} count ={}",
            coin_id, count
        );
        CoinReceiver {
            receiver: sender.subscribe(),
            coin_id: coin_id.to_string(),
            controller: self.clone().into(),
        }
    }
    async fn decrement_receiver_count(&self, coin_id: &str) {
        let mut senders = self.coin_senders.write().await;
        if let Some((_, count)) = senders.get_mut(coin_id) {
            *count -= 1;
            info!("Decrementing count for coin_id: {}", coin_id);
            self.total_channels.fetch_sub(1, Ordering::SeqCst);
            if *count == 0 {
                senders.remove(coin_id);
            }
        }
    }
}

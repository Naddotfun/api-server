use crate::{
    constant::change_channels::{BALANCE, CHART, COIN, CURVE, SWAP, THREAD},
    db::postgres::{controller::info::InfoController, PostgresDatabase},
    types::{
        event::{capture::CoinEventCapture, coin::CoinMessage, SendMessageType},
        model::{
            Balance, BalanceWrapper, ChartWrapper, Coin, Curve, FromValue, Swap, Thread,
            ThreadWrapper,
        },
    },
};
use anyhow::{Context, Result};
use futures::{StreamExt, TryFutureExt};
use serde_json::Value;
use sqlx::postgres::{PgListener, PgNotification};
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
        Self {
            db,
            coin_senders: Arc::new(RwLock::new(HashMap::new())),
            total_channels: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[instrument(skip(self))]
    pub async fn change_data_capture(&self) -> Result<()> {
        let mut listener = PgListener::connect_with(&self.db.pool)
            .await
            .context("Failed to connect to database")?;
        listener
            .listen_all(vec![COIN, SWAP, CHART, BALANCE, CURVE, THREAD])
            .await
            .context("Failed to listen to channels")?;

        let mut stream = listener.into_stream();
        info!("Coin event capture started");

        while let Some(notification) = stream.next().await {
            let producer = self.clone();
            tokio::spawn(async move {
                if let Err(e) = producer.handle_notification(notification).await {
                    error!("Error handling notification: {:?}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_notification(
        &self,
        notification: Result<PgNotification, sqlx::Error>,
    ) -> Result<()> {
        let notification = notification.context("Failed to get notification")?;

        if self.total_channels.load(Ordering::Relaxed) == 0 {
            return Ok(());
        }

        let payload: Value = serde_json::from_str(&notification.payload())
            .context("Failed to parse notification payload")?;

        let coin_id = payload["coin_id"].as_str().unwrap_or("").to_string();

        let event = self.parse_event(notification.channel(), payload)?;
        let senders = self.coin_senders.read().await;
        let should_process = match &event {
            CoinEventCapture::Coin(_) | CoinEventCapture::Swap(_) => true,
            _ => senders.contains_key(&coin_id),
        };

        if !should_process {
            debug!(
                "Skipping event for coin_id: {}, no subscribers or non-relevant event type",
                coin_id
            );
            return Ok(());
        }

        let message = self.handle_event(event).await?;
        // info!("Sending message for coin_id: {:?}\n", message);
        self.send_message(message).await?;

        Ok(())
    }

    fn parse_event(&self, channel: &str, payload: Value) -> Result<CoinEventCapture> {
        info!("Channel = {}, Payload = {:?}", channel, payload);
        match channel {
            COIN => Ok(CoinEventCapture::Coin(Coin::from_value(payload)?)),
            SWAP => Ok(CoinEventCapture::Swap(Swap::from_value(payload)?)),
            CHART => Ok(CoinEventCapture::Chart(ChartWrapper::from_value(payload)?)),
            BALANCE => Ok(CoinEventCapture::Balance(BalanceWrapper::from_value(
                payload,
            )?)),
            CURVE => Ok(CoinEventCapture::Curve(Curve::from_value(payload)?)),
            THREAD => Ok(CoinEventCapture::Thread(ThreadWrapper::from_value(
                payload,
            )?)),
            _ => Err(anyhow::anyhow!("Unknown channel: {}", channel)),
        }
    }

    async fn handle_event(&self, event: CoinEventCapture) -> Result<CoinMessage> {
        let message = match event {
            CoinEventCapture::Coin(coin) => self.handle_coin_event(coin).await,
            CoinEventCapture::Swap(swap) => self.handle_swap_event(swap).await,
            CoinEventCapture::Chart(chart) => self.handle_chart_event(chart).await,
            CoinEventCapture::Balance(balance) => self.handle_balance_event(balance).await,
            CoinEventCapture::Curve(curve) => self.handle_curve_event(curve).await,
            CoinEventCapture::Thread(thread) => self.handle_thread_event(thread).await,
        };

        // info!("Message = {:?}", message);
        message
    }

    async fn handle_coin_event(&self, coin: Coin) -> Result<CoinMessage> {
        let info_controller = InfoController::new(self.db.clone());
        let info = info_controller
            .get_coin_and_user_info(&coin.id, &coin.creator)
            .await?;

        Ok(CoinMessage::from_coin(coin, info))
    }

    async fn handle_swap_event(&self, swap: Swap) -> Result<CoinMessage> {
        let info_controller = InfoController::new(self.db.clone());
        let info = info_controller
            .get_coin_and_user_info(&swap.coin_id, &swap.sender)
            .await?;

        Ok(CoinMessage::from_swap(swap, info))
    }

    async fn handle_chart_event(&self, chart: ChartWrapper) -> Result<CoinMessage> {
        Ok(CoinMessage::from_chart(chart))
    }

    async fn handle_balance_event(&self, balance: BalanceWrapper) -> Result<CoinMessage> {
        info!("Handling balance event: {:?}", balance);
        Ok(CoinMessage::from_balance(balance))
    }

    async fn handle_curve_event(&self, curve: Curve) -> Result<CoinMessage> {
        Ok(CoinMessage::from_curve(curve))
    }

    async fn handle_thread_event(&self, thread: ThreadWrapper) -> Result<CoinMessage> {
        Ok(CoinMessage::from_thread(thread))
    }

    async fn send_message(&self, message: CoinMessage) -> Result<()> {
        let senders = self.coin_senders.read().await;

        match message.message_type {
            SendMessageType::ALL => {
                for (_, sender) in senders.iter() {
                    if let Err(e) = sender.0.send(message.clone()) {
                        error!("Failed to send ALL message: {:?}", e);
                    }
                }
            }
            SendMessageType::Regular => {
                if let Some(sender) = senders.get(&message.coin.id) {
                    if let Err(e) = sender.0.send(message.clone()) {
                        error!(
                            "Failed to send Regular message for coin_id {}: {:?}",
                            message.coin.id, e
                        );
                    }
                } else {
                    warn!("No sender found for coin_id: {}", message.coin.id);
                }
            }
        }

        Ok(())
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

        CoinReceiver {
            receiver: sender.subscribe(),
            coin_id: coin_id.to_string(),
            controller: Arc::new(self.clone()),
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

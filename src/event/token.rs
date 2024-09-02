use crate::{
    constant::change_channels::{BALANCE, CHART, CURVE, SWAP, THREAD, TOKEN},
    db::postgres::{controller::info::InfoController, PostgresDatabase},
    types::{
        event::{capture::TokenEventCapture, token::TokenMessage, SendMessageType},
        model::{
            Balance, BalanceWrapper, ChartWrapper, Curve, FromValue, Swap, Thread, ThreadWrapper,
            Token,
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
pub async fn main(producer: Arc<TokenEventProducer>) -> Result<()> {
    info!("Starting token event capture");

    loop {
        match producer.change_data_capture().await {
            Ok(_) => {
                warn!("Token event capture completed unexpectedly");
                break;
            }
            Err(e) => {
                error!("Error in token change_data_capture: {:?}", e);
                info!("Retrying in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    error!("Token event capture ended");
    Ok(())
}

pub struct TokenReceiver {
    receiver: Receiver<TokenMessage>,
    token_id: String,
    controller: Arc<TokenEventProducer>,
}
impl TokenReceiver {
    pub async fn recv(&mut self) -> Option<TokenMessage> {
        self.receiver.recv().await.ok()
    }
}

impl Drop for TokenReceiver {
    fn drop(&mut self) {
        let controller = self.controller.clone();
        let token_id = self.token_id.clone();
        tokio::spawn(async move {
            controller.decrement_receiver_count(&token_id).await;
        });
    }
}

#[derive(Clone)]
pub struct TokenEventProducer {
    db: Arc<PostgresDatabase>,
    token_senders: Arc<RwLock<HashMap<String, (Sender<TokenMessage>, usize)>>>,
    total_channels: Arc<AtomicUsize>,
}

impl TokenEventProducer {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        Self {
            db,
            token_senders: Arc::new(RwLock::new(HashMap::new())),
            total_channels: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[instrument(skip(self))]
    pub async fn change_data_capture(&self) -> Result<()> {
        let mut listener = PgListener::connect_with(&self.db.pool)
            .await
            .context("Failed to connect to database")?;
        listener
            .listen_all(vec![TOKEN, SWAP, CHART, BALANCE, CURVE, THREAD])
            .await
            .context("Failed to listen to channels")?;

        let mut stream = listener.into_stream();
        info!("Token event capture started");

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

        let token_id = payload["token_id"].as_str().unwrap_or("").to_string();

        let event = self.parse_event(notification.channel(), payload)?;
        let senders = self.token_senders.read().await;
        let should_process = match &event {
            TokenEventCapture::Token(_) | TokenEventCapture::Swap(_) => true,
            _ => senders.contains_key(&token_id),
        };

        if !should_process {
            debug!(
                "Skipping event for token_id: {}, no subscribers or non-relevant event type",
                token_id
            );
            return Ok(());
        }

        let message = self.handle_event(event).await?;
        // info!("Sending message for token_id: {:?}\n", message);
        self.send_message(message).await?;

        Ok(())
    }

    fn parse_event(&self, channel: &str, payload: Value) -> Result<TokenEventCapture> {
        info!("Channel = {}, Payload = {:?}", channel, payload);
        match channel {
            TOKEN => Ok(TokenEventCapture::Token(Token::from_value(payload)?)),
            SWAP => Ok(TokenEventCapture::Swap(Swap::from_value(payload)?)),
            CHART => Ok(TokenEventCapture::Chart(ChartWrapper::from_value(payload)?)),
            BALANCE => Ok(TokenEventCapture::Balance(BalanceWrapper::from_value(
                payload,
            )?)),
            CURVE => Ok(TokenEventCapture::Curve(Curve::from_value(payload)?)),
            THREAD => Ok(TokenEventCapture::Thread(ThreadWrapper::from_value(
                payload,
            )?)),
            _ => Err(anyhow::anyhow!("Unknown channel: {}", channel)),
        }
    }

    async fn handle_event(&self, event: TokenEventCapture) -> Result<TokenMessage> {
        let message = match event {
            TokenEventCapture::Token(token) => self.handle_token_event(token).await,
            TokenEventCapture::Swap(swap) => self.handle_swap_event(swap).await,
            TokenEventCapture::Chart(chart) => self.handle_chart_event(chart).await,
            TokenEventCapture::Balance(balance) => self.handle_balance_event(balance).await,
            TokenEventCapture::Curve(curve) => self.handle_curve_event(curve).await,
            TokenEventCapture::Thread(thread) => self.handle_thread_event(thread).await,
        };

        // info!("Message = {:?}", message);
        message
    }

    async fn handle_token_event(&self, token: Token) -> Result<TokenMessage> {
        let info_controller = InfoController::new(self.db.clone());
        let info = info_controller
            .get_token_and_user_info(&token.id, &token.creator)
            .await?;

        Ok(TokenMessage::from_token(token, info))
    }

    async fn handle_swap_event(&self, swap: Swap) -> Result<TokenMessage> {
        let info_controller = InfoController::new(self.db.clone());
        let info = info_controller
            .get_token_and_user_info(&swap.token_id, &swap.sender)
            .await?;

        Ok(TokenMessage::from_swap(swap, info))
    }

    async fn handle_chart_event(&self, chart: ChartWrapper) -> Result<TokenMessage> {
        Ok(TokenMessage::from_chart(chart))
    }

    async fn handle_balance_event(&self, balance: BalanceWrapper) -> Result<TokenMessage> {
        info!("Handling balance event: {:?}", balance);
        Ok(TokenMessage::from_balance(balance))
    }

    async fn handle_curve_event(&self, curve: Curve) -> Result<TokenMessage> {
        Ok(TokenMessage::from_curve(curve))
    }

    async fn handle_thread_event(&self, thread: ThreadWrapper) -> Result<TokenMessage> {
        Ok(TokenMessage::from_thread(thread))
    }

    async fn send_message(&self, message: TokenMessage) -> Result<()> {
        let senders = self.token_senders.read().await;
        if senders.is_empty() {
            return Ok(());
        }
        for (_, sender) in senders.iter() {
            if let Err(e) = sender.0.send(message.clone()) {
                error!("Failed to send ALL message: {:?}", e);
            }
        }

        // match message.message_type {
        //     SendMessageType::ALL => {
        //         for (_, sender) in senders.iter() {
        //             if let Err(e) = sender.0.send(message.clone()) {
        //                 error!("Failed to send ALL message: {:?}", e);
        //             }
        //         }
        //     }
        //     SendMessageType::Regular => {
        //         if let Some(sender) = senders.get(&message.token.id) {
        //             if let Err(e) = sender.0.send(message.clone()) {
        //                 error!(
        //                     "Failed to send Regular message for token_id {}: {:?}",
        //                     message.token.id, e
        //                 );
        //             }
        //         } else {
        //             warn!("No sender found for token_id: {}", message.token.id);
        //         }
        //     }
        // }

        Ok(())
    }

    pub async fn get_token_receiver(&self, token_id: &str) -> TokenReceiver {
        let mut senders = self.token_senders.write().await;
        let (sender, count) = senders.entry(token_id.to_string()).or_insert_with(|| {
            let new_count = self.total_channels.fetch_add(1, Ordering::SeqCst) + 1;
            info!(
                "Creating new channel for token_id: {}. Total channels: {}",
                token_id, new_count
            );
            (broadcast::channel(1000).0, 0)
        });

        *count += 1;
        info!(
            "Incrementing count for token_id: {}. New count: {}. Total channels: {}",
            token_id,
            count,
            self.total_channels.load(Ordering::SeqCst)
        );

        TokenReceiver {
            receiver: sender.subscribe(),
            token_id: token_id.to_string(),
            controller: Arc::new(self.clone()),
        }
    }

    async fn decrement_receiver_count(&self, token_id: &str) {
        let mut senders = self.token_senders.write().await;
        if let Some((_, count)) = senders.get_mut(token_id) {
            *count -= 1;
            info!(
                "Decremented count for token_id: {}. New count: {}",
                token_id, count
            );
            if *count == 0 {
                senders.remove(token_id);
                info!("Removed channel for token_id: {}", token_id);
            }
        }
        self.total_channels.fetch_sub(1, Ordering::SeqCst);
    }
}

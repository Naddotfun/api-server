use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    constant::change_channels::{COIN, SWAP},
    db::{
        postgres::{
            controller::{info::InfoController, new_content::InitContentController},
            PostgresDatabase,
        },
        redis::RedisDatabase,
    },
    types::{
        event::{
            capture::NewContentCapture, new_content::NewContentMessage, NewSwapMessage,
            NewTokenMessage, SendMessageType,
        },
        model::{
            Coin, CoinReplyCount, CoinReplyCountWrapper, CoinWrapper, Curve, CurveWrapper,
            FromValue, Swap, SwapWrapper,
        },
    },
};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures::StreamExt;

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

#[instrument(skip(producer, redis))]
pub async fn main(producer: Arc<NewContentEventProducer>, redis: Arc<RedisDatabase>) -> Result<()> {
    info!("Starting coin event capture");

    let new_content_controller = InitContentController::new(producer.db.clone());
    {
        let latest_buy = new_content_controller.get_latest_buy().await?;
        let latest_sell = new_content_controller.get_latest_sell().await?;
        let latest_created_coin = new_content_controller.get_latest_new_token().await?;

        redis.set_new_swap(&latest_buy).await?;
        redis.set_new_swap(&latest_sell).await?;
        redis.set_new_token(&latest_created_coin).await?;
    }

    loop {
        if let Err(e) = producer.change_data_capture().await {
            error!("Error in coin change_data_capture: {:?}", e);
            info!("Retrying in 5 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else {
            warn!("Coin event capture completed unexpectedly");
            break;
        }
    }
    error!("Coin event capture ended");
    Ok(())
}
pub struct NewContentReceiver {
    receiver: Receiver<NewContentMessage>,
    controller: Arc<NewContentEventProducer>,
}

impl NewContentReceiver {
    pub async fn recv(&mut self) -> Option<NewContentMessage> {
        self.receiver.recv().await.ok()
    }
}
impl Drop for NewContentReceiver {
    fn drop(&mut self) {
        let controller = self.controller.clone();
        tokio::spawn(async move {
            controller.decrement_receiver_count().await;
        });
    }
}
#[derive(Clone)]
pub struct NewContentEventProducer {
    db: Arc<PostgresDatabase>,
    content_sender: Arc<Sender<NewContentMessage>>,
    total_channels: Arc<AtomicUsize>,
}

impl NewContentEventProducer {
    pub fn new(db: Arc<PostgresDatabase>) -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            db,
            content_sender: Arc::new(sender),
            total_channels: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[instrument(skip(self))]
    pub async fn change_data_capture(&self) -> Result<()> {
        let mut listener = PgListener::connect_with(&self.db.pool).await?;
        listener.listen_all(vec![COIN, SWAP]).await?;
        let mut stream = listener.into_stream();
        info!("New Content event capture started");

        // while let Some(notification) = stream.next().await {
        //     self.handle_notification(notification).await?;
        // }
        while let Some(notification) = stream.next().await {
            let producer = self.clone();
            tokio::spawn(async move {
                if let Err(e) = producer.handle_notification(notification).await {
                    error!("Error handling notification: {:?}", e);
                }
            });
        }

        error!("New Content Changing data capture ended");
        Ok(())
    }

    async fn handle_notification(
        &self,
        notification: Result<PgNotification, sqlx::Error>,
    ) -> Result<()> {
        let notification = notification.context("Failed to get notification")?;

        if self.total_channels.load(Ordering::Relaxed) == 0 {
            debug!("New Content: No active channels");
            return Ok(());
        }

        let payload: Value = serde_json::from_str(&notification.payload())
            .context("Failed to parse notification payload")?;

        let event = match notification.channel() {
            COIN => NewContentCapture::NewToken(Coin::from_value(payload)?),
            SWAP => NewContentCapture::NewSwap(Swap::from_value(payload)?),
            _ => return Ok(()),
        };

        let content_message = self.handle_new_content(event).await?;

        self.send_content_message(content_message).await?;

        Ok(())
    }

    async fn handle_new_content(&self, content: NewContentCapture) -> Result<NewContentMessage> {
        let info_controller = InfoController::new(self.db.clone());
        match content {
            NewContentCapture::NewToken(coin) => {
                let info = info_controller
                    .get_coin_and_user_info(&coin.id, &coin.creator)
                    .await?;
                Ok(NewContentMessage::from_coin(coin, info))
            }
            NewContentCapture::NewSwap(swap) => {
                let info = info_controller
                    .get_coin_and_user_info(&swap.coin_id, &swap.sender)
                    .await?;
                Ok(NewContentMessage::from_swap(swap, info))
            }
        }
    }

    async fn send_content_message(&self, content_message: NewContentMessage) -> Result<()> {
        match self.content_sender.send(content_message) {
            Ok(receiver_count) => {
                debug!("Sent content message to {} receivers", receiver_count);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to send content message: {:?}", e);
                Err(anyhow::anyhow!("Failed to send content message"))
            }
        }
    }

    pub async fn get_content_receiver(&self) -> NewContentReceiver {
        self.total_channels.fetch_add(1, Ordering::SeqCst);
        info!(
            "New Content Receiver count: {}",
            self.total_channels.load(Ordering::SeqCst)
        );
        NewContentReceiver {
            receiver: self.content_sender.subscribe(),
            controller: Arc::new(self.clone()),
        }
    }

    async fn decrement_receiver_count(&self) {
        self.total_channels.fetch_sub(1, Ordering::SeqCst);
        info!(
            "New Content Receiver count: {}",
            self.total_channels.load(Ordering::SeqCst)
        );
    }
}

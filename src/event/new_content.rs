use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    constant::change_channels::{SWAP, TOKEN},
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
            NewTokenMessage,
        },
        model::{FromValue, Swap, Token},
    },
};
use anyhow::{Context, Result};

use futures::StreamExt;

use serde_json::Value;
use sqlx::postgres::{PgListener, PgNotification};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip(producer, redis))]
pub async fn main(producer: Arc<NewContentEventProducer>, redis: Arc<RedisDatabase>) -> Result<()> {
    info!("Starting new_content event capture");

    let new_content_controller = InitContentController::new(producer.db.clone());
    {
        info!("New Content Initializing");

        if let Some(latest_buy) = new_content_controller.get_latest_buy().await? {
            info!("Latest buy: {:?}", latest_buy);
            redis.set_new_swap(&latest_buy).await?;
        } else {
            info!("No latest buy found");
        }

        if let Some(latest_sell) = new_content_controller.get_latest_sell().await? {
            info!("Latest sell: {:?}", latest_sell);
            redis.set_new_swap(&latest_sell).await?;
        } else {
            info!("No latest sell found");
        }

        if let Some(latest_created_token) = new_content_controller.get_latest_new_token().await? {
            info!("Latest created token: {:?}", latest_created_token);
            redis.set_new_token(&latest_created_token).await?;
        } else {
            info!("No latest created token found");
        }
    }
    loop {
        if let Err(e) = producer.change_data_capture().await {
            error!("Error in token change_data_capture: {:?}", e);
            info!("Retrying in 5 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else {
            warn!("Token event capture completed unexpectedly");
            break;
        }
    }
    error!("Token event capture ended");
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
    redis: Arc<RedisDatabase>,
    content_sender: Arc<Sender<NewContentMessage>>,
    total_channels: Arc<AtomicUsize>,
}

impl NewContentEventProducer {
    pub fn new(db: Arc<PostgresDatabase>, redis: Arc<RedisDatabase>) -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            db,
            redis,
            content_sender: Arc::new(sender),
            total_channels: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[instrument(skip(self))]
    pub async fn change_data_capture(&self) -> Result<()> {
        let mut listener = PgListener::connect_with(&self.db.pool).await?;
        listener.listen_all(vec![TOKEN, SWAP]).await?;
        let mut stream = listener.into_stream();
        info!("New Content event capture started");

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

        let payload: Value = serde_json::from_str(&notification.payload())
            .context("Failed to parse notification payload")?;

        let event = match notification.channel() {
            TOKEN => NewContentCapture::NewToken(Token::from_value(payload)?),
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
            NewContentCapture::NewToken(token) => {
                let info = info_controller
                    .get_token_and_user_info(&token.id, &token.creator)
                    .await?;
                let new_token = NewTokenMessage::new(&token, &info);
                self.redis.set_new_token(&new_token).await?;
                Ok(NewContentMessage::from_token(token, info))
            }
            NewContentCapture::NewSwap(swap) => {
                let info = info_controller
                    .get_token_and_user_info(&swap.token_id, &swap.sender)
                    .await?;
                let new_swap = NewSwapMessage::new(&swap, &info);
                self.redis.set_new_swap(&new_swap).await?;
                Ok(NewContentMessage::from_swap(swap, info))
            }
        }
    }

    async fn send_content_message(&self, content_message: NewContentMessage) -> Result<()> {
        let receiver_count = self.total_channels.load(Ordering::Relaxed);
        info!("New Content Receiver count: {}", receiver_count);
        if receiver_count > 0 {
            match self.content_sender.send(content_message) {
                Ok(_) => {
                    debug!("Sent content message to {} receivers", receiver_count);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to send content message: {:?}", e);
                    Err(anyhow::anyhow!("Failed to send content message"))
                }
            }
        } else {
            debug!("No receivers to send content message to");
            Ok(())
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

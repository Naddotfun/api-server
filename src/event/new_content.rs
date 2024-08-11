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
        postgres::{controller::info::InfoController, PostgresDatabase},
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
pub async fn main(producer: Arc<NewContentEventProducer>) -> Result<()> {
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
        info!("New Content event  capture Start");

        while let Some(notification) = stream.next().await {
            if let Ok(notification) = notification {
                if self.total_channels.load(Ordering::Relaxed) == 0 {
                    debug!("New Content No active channels");
                    continue; // 총 채널 수가 0이면 다음 알림으로 넘어갑니다.
                }

                let payload: Value = serde_json::from_str(&notification.payload())?;

                let event = match notification.channel() {
                    COIN => NewContentCapture::NewToken(Coin::from_value(payload)?),
                    SWAP => NewContentCapture::NewSwap(Swap::from_value(payload)?),
                    _ => continue,
                };

                match self.handle_new_content(event).await {
                    Ok(content_message) => {
                        debug!("Handled event successfully");
                        match self.content_sender.send(content_message) {
                            Ok(receiver_count) => {
                                debug!("Sent content message to {} receivers", receiver_count);
                            }
                            Err(e) => {
                                warn!("Failed to send content message: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to handle event: {:?}", e);
                        continue;
                    }
                }
            }
        }
        error!("New Content Changing data capture end");
        Ok(())
    }

    pub async fn handle_new_content(
        &self,
        content: NewContentCapture,
    ) -> Result<NewContentMessage> {
        match content {
            NewContentCapture::NewToken(coin) => {
                let info_controller = InfoController::new(self.db.clone());
                let info = info_controller
                    .get_coin_and_user_info(&coin.id, &coin.creator)
                    .await?;
                Ok(NewContentMessage::from_coin(coin, info))
            }
            NewContentCapture::NewSwap(swap) => {
                let info_controller = InfoController::new(self.db.clone());
                let info = info_controller
                    .get_coin_and_user_info(&swap.coin_id, &swap.sender)
                    .await?;
                Ok(NewContentMessage::from_swap(swap, info))
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
            controller: self.clone().into(),
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

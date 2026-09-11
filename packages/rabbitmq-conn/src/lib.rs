use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use lapin::options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RabbitMQClient {
    conn: Arc<RwLock<Option<Connection>>>,
}

pub fn get_chunk_queue() -> String {
    std::env::var("QUEUE_CHUNK").unwrap_or_else(|_| "chunk-queue".to_string())
}

pub fn get_transcode_queue() -> String {
    std::env::var("QUEUE_TRANSCODE").unwrap_or_else(|_| "transcode-jobs".to_string())
}

pub fn get_db_updates_queue() -> String {
    std::env::var("QUEUE_DB_UPDATES").unwrap_or_else(|_| "db-updates".to_string())
}

pub fn get_merge_queue() -> String {
    std::env::var("QUEUE_MERGE").unwrap_or_else(|_| "merge-jobs".to_string())
}

pub fn get_dlq() -> String {
    std::env::var("QUEUE_DLQ").unwrap_or_else(|_| "dead-letter-queue".to_string())
}

impl RabbitMQClient {
    /// Creates a new RabbitMQ client that connects in the background.
    /// Returns immediately without blocking.
    pub fn new(uri: &str) -> Self {
        // Ensure crypto provider is installed for RabbitMQ's rustls backend
        let _ = rustls::crypto::ring::default_provider().install_default();

        let client = Self {
            conn: Arc::new(RwLock::new(None)),
        };

        let conn_state = client.conn.clone();
        let uri = uri.to_string();

        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            let max_backoff = Duration::from_secs(30);

            loop {
                log::info!("Attempting to connect to RabbitMQ...");
                match Connection::connect(&uri, ConnectionProperties::default()).await {
                    Ok(conn) => {
                        log::info!("Successfully connected to RabbitMQ");
                        
                        // Auto-declare essential queues upon successful connection
                        if let Ok(channel) = conn.create_channel().await {
                            let _ = Self::declare_standard_queues(&channel).await;
                        }

                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                        conn.on_error(move |error| {
                            log::error!("RabbitMQ connection error: {:?}", error);
                            let _ = tx.send(());
                        });

                        *conn_state.write().await = Some(conn);
                        
                        // Wait for connection to drop
                        rx.recv().await;
                        
                        log::warn!("RabbitMQ connection closed. Reconnecting...");
                        *conn_state.write().await = None;
                        backoff = Duration::from_secs(1);
                    }
                    Err(e) => {
                        log::error!("Failed to connect to RabbitMQ: {}. Retrying in {} seconds...", e, backoff.as_secs());
                        tokio::time::sleep(backoff).await;
                        backoff = std::cmp::min(backoff * 2, max_backoff);
                    }
                }
            }
        });

        client
    }

    /// Creates a new RabbitMQ client and blocks until the first successful connection.
    pub async fn new_and_wait(uri: &str) -> Self {
        let client = Self::new(uri);
        loop {
            if client.conn.read().await.is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        client
    }

    /// Creates a new channel.
    pub async fn create_channel(&self) -> Result<Channel> {
        let conn_guard = self.conn.read().await;
        let conn = conn_guard.as_ref().ok_or_else(|| anyhow!("RabbitMQ not connected yet"))?;
        let channel = conn.create_channel().await.context("Failed to create channel")?;
        Ok(channel)
    }

    /// Declares a queue.
    pub async fn declare_queue(&self, channel: &Channel, queue_name: &str) -> Result<()> {
        let mut options = QueueDeclareOptions::default();
        options.durable = true;
        
        let _: lapin::Queue = channel
            .queue_declare(
                queue_name,
                options,
                FieldTable::default(),
            )
            .await
            .context("Failed to declare queue")?;
        Ok(())
    }

    /// Declares all standard queues used in the transcoding pipeline.
    pub async fn declare_standard_queues(channel: &Channel) -> Result<()> {
        let chunk_queue = get_chunk_queue();
        let transcode_queue = get_transcode_queue();
        let db_updates = get_db_updates_queue();
        let merge_jobs = get_merge_queue();
        let dlq = get_dlq();

        let queues = vec![
            chunk_queue,
            transcode_queue,
            db_updates,
            merge_jobs,
            dlq,
        ];

        for q in queues {
            let mut options = QueueDeclareOptions::default();
            options.durable = true;
            channel
                .queue_declare(
                    &q,
                    options,
                    FieldTable::default(),
                )
                .await
                .context(format!("Failed to auto-declare queue: {}", q))?;
        }
        Ok(())
    }

    /// Publishes a message to a queue/exchange.
    pub async fn publish(
        &self,
        channel: &Channel,
        exchange: &str,
        routing_key: &str,
        payload: &[u8],
    ) -> Result<()> {
        let _ = channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await
            .context("Failed to publish message")?;
        Ok(())
    }

    /// Consumes messages from a queue and passes them to a handler callback.
    pub async fn consume<F>(&self, channel: &Channel, queue_name: &str, consumer_tag: &str, handler: F) -> Result<()>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let mut consumer = channel
            .basic_consume(
                queue_name,
                consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .context("Failed to start basic consume")?;

        tokio::spawn(async move {
            while let Some(delivery) = consumer.next().await {
                if let Ok(delivery) = delivery {
                    handler(delivery.data.clone());
                    let _ = delivery.ack(lapin::options::BasicAckOptions::default()).await;
                }
            }
        });

        Ok(())
    }
}

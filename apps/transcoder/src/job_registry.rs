use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

#[derive(Clone)]
pub struct JobRegistry {
    // Maps video_id -> HashMap<task_id, cancel_tx>
    jobs: Arc<Mutex<HashMap<i32, HashMap<String, oneshot::Sender<()>>>>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers an active transcode task's cancel channel for a given video ID and unique task identifier.
    pub async fn register(&self, video_id: i32, task_key: String, cancel_tx: oneshot::Sender<()>) {
        let mut map = self.jobs.lock().await;
        map.entry(video_id).or_default().insert(task_key, cancel_tx);
    }

    /// Triggers cancellation signals on all active task senders for a video ID.
    pub async fn cancel(&self, video_id: i32) {
        let mut map = self.jobs.lock().await;
        if let Some(tasks) = map.remove(&video_id) {
            log::warn!("Cancelling all active transcoding chunks for video_id: {}", video_id);
            for (_key, tx) in tasks {
                let _ = tx.send(()); // Trigger abort inside tokio::select!
            }
        }
    }

    /// Removes finished task channel mapping from the registry.
    pub async fn unregister(&self, video_id: i32, task_key: &str) {
        let mut map = self.jobs.lock().await;
        if let Some(tasks) = map.get_mut(&video_id) {
            tasks.remove(task_key);
            if tasks.is_empty() {
                map.remove(&video_id);
            }
        }
    }
}
pub type SharedRegistry = JobRegistry;

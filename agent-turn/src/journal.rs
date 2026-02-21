use std::sync::Arc;

use agent_core::TranscriptItem;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct TranscriptJournal {
    inner: Arc<Mutex<Vec<TranscriptItem>>>,
}

impl TranscriptJournal {
    pub async fn append(&self, items: &[TranscriptItem]) {
        if items.is_empty() {
            return;
        }
        let mut guard = self.inner.lock().await;
        guard.extend(items.iter().cloned());
    }

    pub async fn all(&self) -> Vec<TranscriptItem> {
        self.inner.lock().await.clone()
    }
}

use super::{
    chunks::{ChunksInfo, DownloadParts, NonResumablePart, ResumablePart},
    download::Download,
};
use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::Mutex;

pub struct DownloadHandle {
    pub core: Download,
    pub chunks_progress: ChunksProgress,
    pub last_update_time: DateTime<Utc>,
    pub task_handles: Vec<tokio::task::JoinHandle<()>>,
    pub pause_tx: tokio::sync::watch::Sender<bool>,
}

impl DownloadHandle {
    pub async fn pause(self) {
        let _ = self.pause_tx.send(true);
        let _ = join_all(self.task_handles).await;
    }

    pub async fn update_progress(&mut self) {
        match &self.chunks_progress {
            ChunksProgress::Resumable(parts) => {
                let mut unlokced = vec![];
                for part in parts {
                    let p = part.lock().await;
                    unlokced.push(p.clone());
                }
                if let ChunksInfo::InfoLoaded(loaded) = &mut self.core.chunks_info {
                    loaded.parts = DownloadParts::Resumable(unlokced)
                }
            }
            ChunksProgress::NonResumable(part) => {
                if let ChunksInfo::InfoLoaded(loaded) = &mut self.core.chunks_info {
                    let unlocked = { (part.lock().await).clone() };
                    loaded.parts = DownloadParts::NonResumable(unlocked)
                }
            }
        }
    }
}

pub enum ChunksProgress {
    Resumable(Vec<Arc<Mutex<ResumablePart>>>),
    NonResumable(Arc<Mutex<NonResumablePart>>),
}

impl Deref for DownloadHandle {
    type Target = Download;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl DerefMut for DownloadHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

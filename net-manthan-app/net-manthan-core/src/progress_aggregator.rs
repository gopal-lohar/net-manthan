use crate::download_part::ChunkProgress;
use crate::DownloadProgress;
use chrono::{Duration, Utc};
use crossbeam_channel::Receiver;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration as StdDuration,
};
use tokio::sync::broadcast;

// TODO: close this thread when the download is complete
pub async fn progress_aggregator(
    download_id: u64,
    progress_receiver: Receiver<ChunkProgress>,
    broadcast_sender: broadcast::Sender<DownloadProgress>,
    udpate_interval: Duration,
    cancel_token: Arc<AtomicBool>,
) {
    let mut download_progress: DownloadProgress = DownloadProgress {
        download_id,
        chunks: HashMap::<u32, ChunkProgress>::new(),
    };

    let mut last_update = Utc::now();

    while !cancel_token.load(Ordering::Relaxed) {
        match progress_receiver.recv_timeout(StdDuration::from_millis(100)) {
            Ok(chunk_progress) => {
                download_progress
                    .chunks
                    .insert(chunk_progress.chunk_id, chunk_progress);
            }
            Err(_) => {}
        }

        if (Utc::now() - last_update).num_milliseconds() > udpate_interval.num_milliseconds() as i64
        {
            if broadcast_sender.send(download_progress.clone()).is_err() {
                break;
            }
            last_update = Utc::now();
        }
    }
}

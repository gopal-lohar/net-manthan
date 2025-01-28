use crate::download_part::ChunkProgress;
use crate::DownloadProgress;
use crossbeam_channel::Receiver;
use tokio::sync::broadcast;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

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

    let mut last_update = Instant::now();
    while !cancel_token.load(Ordering::Relaxed) {
        match progress_receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(chunk_progress) => {
                download_progress
                    .chunks
                    .insert(chunk_progress.chunk_id, chunk_progress);
            }
            Err(_) => {}
        }
        if last_update.elapsed() > udpate_interval {
            if broadcast_sender.send(download_progress.clone()).is_err() {
                break;
            }
            last_update = Instant::now();
        }
    }
}

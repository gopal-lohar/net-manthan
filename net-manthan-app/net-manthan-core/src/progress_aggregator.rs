use crate::download_part::ChunkProgress;
use crossbeam_channel::Receiver;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tracing::debug;

struct DownloadProgress {
    download_id: u64,
    chunks: HashMap<u32, ChunkProgress>,
}

pub fn progress_aggregator(
    download_id: u64,
    progress_receiver: Receiver<ChunkProgress>,
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
            debug!("update: {}", download_progress.download_id);
            last_update = Instant::now();
        }
    }
}

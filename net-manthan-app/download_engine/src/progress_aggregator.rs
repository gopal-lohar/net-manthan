use crate::types::{DownloadStatus, PartProgress};
use chrono::{Duration, Utc};
use crossbeam_channel::Sender;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::Mutex;

pub async fn progress_aggregator(
    part_progress_vec: Vec<Arc<Mutex<PartProgress>>>,
    aggregator_sender: Sender<Vec<PartProgress>>,
    update_interval: Duration,
    cancel_token: Arc<AtomicBool>,
) {
    let mut last_update = Utc::now();

    while !cancel_token.load(Ordering::Relaxed) {
        if (Utc::now() - last_update) > update_interval {
            let mut exit_thread = true;
            if aggregator_sender
                .send({
                    let mut vec = Vec::new();
                    for part in &part_progress_vec {
                        let guard = part.lock().await;
                        let something_going_on = match guard.status {
                            DownloadStatus::Queued => true,
                            DownloadStatus::Connecting => true,
                            DownloadStatus::Downloading => true,
                            DownloadStatus::Paused => false,
                            DownloadStatus::Completed(_) => false,
                            DownloadStatus::Failed(_) => false,
                            DownloadStatus::Cancelled => false,
                        };
                        if something_going_on {
                            exit_thread = false;
                        }
                        vec.push(guard.clone());
                    }
                    vec
                })
                .is_err()
                || exit_thread
            {
                break;
            } else {
                last_update = Utc::now();
            }
        }
    }
}

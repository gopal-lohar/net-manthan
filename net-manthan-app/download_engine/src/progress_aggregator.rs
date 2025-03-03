use crate::types::PartProgress;
use chrono::{Duration, Utc};
use crossbeam_channel::{Receiver, Sender};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration as StdDuration,
};

// TODO: close this thread when the download is complete
pub async fn progress_aggregator(
    mut download_progress: Vec<PartProgress>,
    progress_receiver: Receiver<PartProgress>,
    aggregator_sender: Sender<Vec<PartProgress>>,
    udpate_interval: Duration,
    cancel_token: Arc<AtomicBool>,
) {
    let mut last_update = Utc::now();

    while !cancel_token.load(Ordering::Relaxed) {
        match progress_receiver.recv_timeout(StdDuration::from_millis(100)) {
            Ok(part_progress) => {
                let part_id = part_progress.part_id.clone();
                if let Some(x) = download_progress.iter_mut().find(|x| x.part_id == part_id) {
                    *x = part_progress.clone();
                }
            }
            Err(_) => {}
        }

        if (Utc::now() - last_update).num_milliseconds() > udpate_interval.num_milliseconds() as i64
        {
            if aggregator_sender.send(download_progress.clone()).is_err() {
                break;
            }
            last_update = Utc::now();
        }
    }
}

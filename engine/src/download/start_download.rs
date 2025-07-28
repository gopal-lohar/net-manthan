use crate::types::{
    chunks::{ChunksInfo, DownloadParts},
    download::Download,
    download_handle::{ChunksProgress, DownloadHandle},
    status::DownloadStatus,
};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

use super::download_part::download_part;

// we won't fetch the info in this function or it's descendants because
// we don't want to wrap the download in mutex

// we don't want to create the file because fetching info, creating download parts
//  and creating the file is a one time task while downloading the file can be resumed

/// start a download, make sure the download file and the download info is already loaded
pub async fn start_download(download: Download) -> Option<DownloadHandle> {
    match &download.chunks_info {
        ChunksInfo::InfoLoaded(info_loaded) => {
            let (pause_tx, _) = watch::channel(false);

            let mut task_handles = vec![];
            let chunks_progress = match &info_loaded.parts {
                DownloadParts::Resumable(parts) => {
                    let mut progress_vec = vec![];
                    for part in parts {
                        let mut part = part.clone();
                        part.status = DownloadStatus::Downloading;
                        let request = download.request.clone();
                        let info = info_loaded.info.clone();
                        let config = download.config.clone();
                        let pause_rx = pause_tx.subscribe();
                        let progress = Arc::new(Mutex::new(part.clone()));
                        progress_vec.push(Arc::clone(&progress));
                        task_handles.push(tokio::task::spawn(async move {
                            download_part(request, info, part, progress, config, pause_rx).await;
                        }));
                    }
                    ChunksProgress::Resumable(progress_vec)
                }
                DownloadParts::NonResumable(part) => {
                    let mut part = part.clone();
                    part.status = DownloadStatus::Downloading;
                    let request = download.request.clone();
                    let info = info_loaded.info.clone();
                    let config = download.config.clone();
                    let pause_rx = pause_tx.subscribe();
                    let progress = Arc::new(Mutex::new(part.clone()));
                    let progress_clone = Arc::clone(&progress);
                    task_handles.push(tokio::task::spawn(async move {
                        download_part(request, info, part, progress, config, pause_rx).await;
                    }));
                    ChunksProgress::NonResumable(progress_clone)
                }
            };

            Some(DownloadHandle {
                core: download,
                chunks_progress,
                last_update_time: Utc::now(),
                task_handles,
                pause_tx,
            })
        }
        _ => None,
    }
}

use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use crate::rpc_types::{
    Download as DownloadProto, DownloadRequest, DownloadStatus as DownloadStatusProto,
    NonResumablePart as NoneRseumablePartProto, None as NoneProto,
    ResumablePart as ResumablePartProto, ResumableParts as ResumablePartsProto,
    download::Parts as PartsProto,
};
use download_engine::{
    Download, DownloadParts, NonResumableDownloadPart, ResumableDownloadPart,
    download_config::DownloadConfig, types::DownloadStatus,
};
use uuid::Uuid;

use chrono::{DateTime, Duration as ChronoDuration, TimeZone, Utc};
use prost_types::Timestamp;

use prost_types::Duration as ProtoDuration;

pub fn convert_to_timestamp_proto(dt: &DateTime<Utc>) -> Timestamp {
    let duration_since_epoch = dt.signed_duration_since(DateTime::<Utc>::UNIX_EPOCH);
    let total_nanos = duration_since_epoch
        .num_nanoseconds()
        .expect("DateTime out of range");
    let seconds = total_nanos / 1_000_000_000;
    let nanos = (total_nanos % 1_000_000_000) as i32;

    // Ensure nanos are non-negative (required by Protobuf)
    if nanos < 0 {
        Timestamp {
            seconds: seconds - 1,
            nanos: nanos + 1_000_000_000,
        }
    } else {
        Timestamp { seconds, nanos }
    }
}

pub fn convert_to_duration_proto(d: &ChronoDuration) -> ProtoDuration {
    let total_nanos = d.num_nanoseconds().expect("Duration out of range");
    let seconds = total_nanos / 1_000_000_000;
    let nanos = (total_nanos % 1_000_000_000) as i32;

    // Normalize signs to match Protobuf requirements
    if seconds > 0 && nanos < 0 {
        ProtoDuration {
            seconds: seconds - 1,
            nanos: nanos + 1_000_000_000,
        }
    } else if seconds < 0 && nanos > 0 {
        ProtoDuration {
            seconds: seconds + 1,
            nanos: nanos - 1_000_000_000,
        }
    } else {
        ProtoDuration { seconds, nanos }
    }
}

pub fn convert_from_timestamp_proto(timestamp: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(timestamp.seconds, timestamp.nanos as u32)
        .single()
        .unwrap_or_else(|| {
            // Fallback to Unix epoch for invalid timestamps
            Utc.timestamp_opt(0, 0).single().unwrap()
        })
}

pub fn convert_from_duration_proto(duration: ProtoDuration) -> ChronoDuration {
    // Convert seconds and nanoseconds separately
    let seconds = duration.seconds.max(0) as i64;
    let nanos = duration.nanos.max(0) as i64;

    // ChronoDuration can handle large values and negative durations
    ChronoDuration::seconds(seconds) + ChronoDuration::nanoseconds(nanos)
}

pub fn convert_to_download_req(req: DownloadRequest) -> download_engine::types::DownloadRequest {
    download_engine::types::DownloadRequest {
        url: req.url,
        file_dir: PathBuf::from(req.file_dir),
        file_name: req.filename.map(PathBuf::from),
        referrer: req.referrer,
        headers: Some(req.headers),
    }
}

pub fn convert_to_download_req_proto(
    req: download_engine::types::DownloadRequest,
) -> DownloadRequest {
    DownloadRequest {
        url: req.url,
        file_dir: req
            .file_dir
            .to_str()
            .map(|p| p.to_string())
            .unwrap_or("default-filename".to_string()),
        filename: req.file_name.map(|n| {
            n.to_str()
                .map(|p| p.to_string())
                .unwrap_or("default-filename".to_string())
        }),
        referrer: req.referrer,
        headers: req.headers.unwrap_or(vec![]),
    }
}

pub fn convert_to_download_proto(download: &Download) -> DownloadProto {
    DownloadProto {
        id: download.id.to_string(),
        url: download.url.to_owned(),
        // TODO: fix this entire filename thing
        file: download
            .file
            .clone()
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or("Default File Name".to_string()),
        file_name: download
            .file_name
            .clone()
            .map(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or(Some("Default File Name".to_string()))
            .unwrap_or("Default File Name".to_string()),
        headers: download.headers.to_owned().unwrap_or(vec![]),
        referrer: download.referrer.clone(),
        date_added: Some(convert_to_timestamp_proto(&download.date_added)),
        active_time: Some(convert_to_duration_proto(&download.active_time)),
        status: convert_to_download_status_proto(&download.get_status()) as i32,
        parts: Some(match &download.parts {
            DownloadParts::NonResumable(part) => PartsProto::NonResumable(NoneRseumablePartProto {
                id: part.id.to_string(),
                status: convert_to_download_status_proto(&part.status) as i32,
                total_bytes: part.total_size,
                bytes_downloaded: part.bytes_downloaded,
                current_speed: part.current_speed as u64,
            }),
            DownloadParts::Resumable(parts) => PartsProto::Resumable(ResumablePartsProto {
                parts: parts
                    .iter()
                    .map(|p| ResumablePartProto {
                        id: p.id.to_string(),
                        status: convert_to_download_status_proto(&p.status) as i32,
                        start_byte: p.start_byte,
                        end_byte: p.end_byte,
                        bytes_downloaded: p.bytes_downloaded,
                        current_speed: p.current_speed as u64,
                    })
                    .collect(),
            }),
            DownloadParts::None => PartsProto::None(NoneProto {}),
        }),
    }
}

pub fn convert_from_download_proto(download: &DownloadProto) -> Download {
    Download {
        id: Uuid::parse_str(&download.id).expect("Invalid UUID"),
        url: download.url.to_owned(),
        // TODO: fix this entire filename thing
        file: PathBuf::from(download.file.clone()),
        file_name: Some(PathBuf::from(download.file_name.clone())),
        headers: (!download.headers.is_empty()).then_some(download.headers.clone()),
        referrer: download.referrer.clone(),
        date_added: download
            .date_added
            .map(convert_from_timestamp_proto)
            .expect("Missing date_added"),
        active_time: download
            .active_time
            .map(convert_from_duration_proto)
            .expect("Missing active_time"),

        status: convert_from_download_status_proto(&download.status()),
        parts: match download
            .parts
            .clone()
            .unwrap_or(PartsProto::None(NoneProto {}))
        {
            PartsProto::NonResumable(part) => {
                DownloadParts::NonResumable(NonResumableDownloadPart {
                    id: Uuid::parse_str(&part.id).unwrap_or(Uuid::new_v4()),
                    status: convert_from_download_status_proto(&part.status()),
                    total_size: part.total_bytes,
                    bytes_downloaded: part.bytes_downloaded,
                    current_speed: part.current_speed as usize,
                })
            }
            PartsProto::Resumable(list) => DownloadParts::Resumable(
                list.parts
                    .iter()
                    .map(|part| ResumableDownloadPart {
                        id: Uuid::parse_str(&part.id).unwrap_or(Uuid::new_v4()),
                        status: convert_from_download_status_proto(&part.status()),
                        start_byte: part.start_byte,
                        end_byte: part.end_byte,
                        bytes_downloaded: part.bytes_downloaded,
                        current_speed: part.current_speed as usize,
                    })
                    .collect::<Vec<ResumableDownloadPart>>(),
            ),
            PartsProto::None(_) => DownloadParts::None,
        },
        // lost after going through protobuff
        config: DownloadConfig::default(),
        last_update_time: None,
        progress: download_engine::DownloadPartsProgress::None,
        stop_token: Arc::new(AtomicBool::new(false)),
    }
}

pub fn convert_to_download_status_proto(status: &DownloadStatus) -> DownloadStatusProto {
    match status {
        DownloadStatus::Created => DownloadStatusProto::Created,
        DownloadStatus::Queued => DownloadStatusProto::Queued,
        DownloadStatus::Connecting => DownloadStatusProto::Connecting,
        DownloadStatus::Retrying => DownloadStatusProto::Retrying,
        DownloadStatus::Downloading => DownloadStatusProto::Downloading,
        DownloadStatus::Paused => DownloadStatusProto::Paused,
        DownloadStatus::Complete => DownloadStatusProto::Complete,
        DownloadStatus::Failed => DownloadStatusProto::Failed,
        DownloadStatus::Cancelled => DownloadStatusProto::Cancelled,
    }
}

pub fn convert_from_download_status_proto(status: &DownloadStatusProto) -> DownloadStatus {
    match status {
        DownloadStatusProto::StatusUnspecified => DownloadStatus::Created,
        DownloadStatusProto::Created => DownloadStatus::Created,
        DownloadStatusProto::Queued => DownloadStatus::Queued,
        DownloadStatusProto::Connecting => DownloadStatus::Connecting,
        DownloadStatusProto::Retrying => DownloadStatus::Retrying,
        DownloadStatusProto::Downloading => DownloadStatus::Downloading,
        DownloadStatusProto::Paused => DownloadStatus::Paused,
        DownloadStatusProto::Complete => DownloadStatus::Complete,
        DownloadStatusProto::Failed => DownloadStatus::Failed,
        DownloadStatusProto::Cancelled => DownloadStatus::Cancelled,
    }
}

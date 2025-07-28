use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeStamps {
    pub date_added: DateTime<Utc>,
    pub active_time: Duration,
    pub date_updated: DateTime<Utc>,
    pub date_completed: Option<DateTime<Utc>>,
}

impl TimeStamps {
    pub fn now() -> Self {
        TimeStamps {
            date_added: Utc::now(),
            active_time: Duration::zero(),
            date_updated: Utc::now(),
            date_completed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub buffer_size: usize,
    pub update_interval: usize,
    pub retry_count: usize,
    pub connections_per_server: usize,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024 * 256,
            update_interval: 500,
            retry_count: 3,
            connections_per_server: 2,
        }
    }
}

#[derive(Debug)]
pub struct DownloadConfig {
    pub buffer_size: usize,
    pub update_interval: usize,
    pub retry_count: usize,
    pub connections_per_server: usize,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            update_interval: 500,
            retry_count: 3,
            connections_per_server: 10,
        }
    }
}

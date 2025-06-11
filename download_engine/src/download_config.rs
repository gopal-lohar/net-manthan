#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub buffer_size: usize,
    pub update_interval: usize,
    pub retry_count: usize,
    pub connections_per_server: usize,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            // at buffer size 1024 * 100, download rate limited to 500KB/s in ngix
            // a 50MB file is getting downloaded 50.59MB while if buffer size increased to 1024*1024 no problem
            buffer_size: 1024 * 50,
            update_interval: 500,
            retry_count: 3,
            connections_per_server: 10,
        }
    }
}
